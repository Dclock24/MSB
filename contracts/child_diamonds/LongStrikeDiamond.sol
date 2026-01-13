// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title LongStrikeDiamond
 * @notice Child Diamond managing 25 Long Strike Bots
 * @dev Layer 1 Child Diamond for Long positions
 */

import {LibDiamond} from "../libraries/LibDiamond.sol";
import {IDiamondCut} from "../interfaces/IDiamondCut.sol";
import {IChildDiamond} from "../interfaces/IChildDiamond.sol";
import {ILongStrikeFacet} from "../interfaces/ILongStrikeFacet.sol";

contract LongStrikeDiamond is IChildDiamond {
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

    /**
     * @notice Execute operation from master diamond
     */
    function executeOperation(
        uint8 _operation,
        bytes memory _data
    ) external onlyMaster returns (uint256) {
        // Route to appropriate facet based on operation
        if (_operation == uint8(OperationType.Strike)) {
            return _executeLongStrike(_data);
        } else if (_operation == uint8(OperationType.Rebalance)) {
            return _rebalanceLongBots(_data);
        }
        revert("Unknown operation");
    }

    function _executeLongStrike(bytes memory _data) internal returns (uint256) {
        // Decode strike opportunity
        ILongStrikeFacet.StrikeOpportunity memory opportunity = abi.decode(
            _data,
            (ILongStrikeFacet.StrikeOpportunity)
        );
        
        // Call LongStrikeFacet
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        address facet = ds.selectorToFacetAndPosition[
            ILongStrikeFacet.executeLongStrike.selector
        ].facetAddress;
        
        require(facet != address(0), "LongStrikeFacet not found");
        
        (bool success, bytes memory result) = facet.delegatecall(
            abi.encodeWithSignature(
                "executeLongStrike((uint8,uint256,address,uint256,uint256,uint256))",
                opportunity
            )
        );
        
        require(success, "Long strike execution failed");
        (bool strikeSuccess, uint256 profit) = abi.decode(result, (bool, uint256));
        return profit;
    }

    function _rebalanceLongBots(bytes memory _data) internal returns (uint256) {
        // Call rebalance function
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        address facet = ds.selectorToFacetAndPosition[
            ILongStrikeFacet.rebalanceLongBots.selector
        ].facetAddress;
        
        require(facet != address(0), "LongStrikeFacet not found");
        
        (bool success, bytes memory result) = facet.delegatecall(
            abi.encodeWithSignature("rebalanceLongBots()")
        );
        
        require(success, "Rebalance failed");
        return abi.decode(result, (uint256));
    }

    /**
     * @notice Get statistics for master diamond
     */
    function getStats() external view returns (StrikeStats memory) {
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        address facet = ds.selectorToFacetAndPosition[
            ILongStrikeFacet.getLongStrikeStats.selector
        ].facetAddress;
        
        require(facet != address(0), "LongStrikeFacet not found");
        
        (bool success, bytes memory result) = facet.staticcall(
            abi.encodeWithSignature("getLongStrikeStats()")
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
