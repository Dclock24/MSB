// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ShortStrikeDiamond
 * @notice Child Diamond managing 25 Short Strike Bots
 * @dev Layer 1 Child Diamond for Short positions
 */

import {LibDiamond} from "../libraries/LibDiamond.sol";
import {IDiamondCut} from "../interfaces/IDiamondCut.sol";
import {IChildDiamond} from "../interfaces/IChildDiamond.sol";
import {IShortStrikeFacet} from "../interfaces/IShortStrikeFacet.sol";

contract ShortStrikeDiamond is IChildDiamond {
    address public masterDiamond;
    
    constructor(
        address _masterDiamond,
        address _contractOwner,
        address _diamondCutFacet
    ) {
        masterDiamond = _masterDiamond;
        LibDiamond.setContractOwner(_contractOwner);

        IDiamondCut.FacetCut[] memory cut = new IDiamondCut.FacetCut[](1);
        bytes4[] memory functionSelectors = new bytes4[](1);
        functionSelectors[0] = IDiamondCut.diamondCut.selector;
        cut[0] = IDiamondCut.FacetCut({
            facetAddress: _diamondCutFacet,
            action: IDiamondCut.FacetCutAction.Add,
            functionSelectors: functionSelectors
        });
        LibDiamond.diamondCut(cut, address(0), "");
    }

    modifier onlyMaster() {
        require(msg.sender == masterDiamond, "Only master diamond");
        _;
    }

    function executeOperation(
        uint8 _operation,
        bytes memory _data
    ) external onlyMaster returns (uint256) {
        if (_operation == uint8(OperationType.Strike)) {
            return _executeShortStrike(_data);
        } else if (_operation == uint8(OperationType.Rebalance)) {
            return _rebalanceShortBots(_data);
        }
        revert("Unknown operation");
    }

    function _executeShortStrike(bytes memory _data) internal returns (uint256) {
        IShortStrikeFacet.StrikeOpportunity memory opportunity = abi.decode(
            _data,
            (IShortStrikeFacet.StrikeOpportunity)
        );
        
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        address facet = ds.selectorToFacetAndPosition[
            IShortStrikeFacet.executeShortStrike.selector
        ].facetAddress;
        
        require(facet != address(0), "ShortStrikeFacet not found");
        
        (bool success, bytes memory result) = facet.delegatecall(
            abi.encodeWithSignature(
                "executeShortStrike((uint8,uint8,uint256,address,uint256,uint256,uint256))",
                opportunity
            )
        );
        
        require(success, "Short strike execution failed");
        (bool strikeSuccess, uint256 profit) = abi.decode(result, (bool, uint256));
        return profit;
    }

    function _rebalanceShortBots(bytes memory _data) internal returns (uint256) {
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        address facet = ds.selectorToFacetAndPosition[
            IShortStrikeFacet.rebalanceShortBots.selector
        ].facetAddress;
        
        require(facet != address(0), "ShortStrikeFacet not found");
        
        (bool success, bytes memory result) = facet.delegatecall(
            abi.encodeWithSignature("rebalanceShortBots()")
        );
        
        require(success, "Rebalance failed");
        return abi.decode(result, (uint256));
    }

    function getStats() external view returns (StrikeStats memory) {
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        address facet = ds.selectorToFacetAndPosition[
            IShortStrikeFacet.getShortStrikeStats.selector
        ].facetAddress;
        
        require(facet != address(0), "ShortStrikeFacet not found");
        
        (bool success, bytes memory result) = facet.staticcall(
            abi.encodeWithSignature("getShortStrikeStats()")
        );
        
        require(success, "Get stats failed");
        return abi.decode(result, (StrikeStats));
    }

    fallback() external payable {
        LibDiamond.DiamondStorage storage ds;
        bytes32 position = LibDiamond.DIAMOND_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
        address facet = ds.selectorToFacetAndPosition[msg.sig].facetAddress;
        require(facet != address(0), "Diamond: Function does not exist");
        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), facet, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result
            case 0 {
                revert(0, returndatasize())
            }
            default {
                return(0, returndatasize())
            }
        }
    }

    receive() external payable {}
}
