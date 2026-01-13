// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title MasterDiamond
 * @notice Master Diamond overseeing 3 Layer Children Diamonds
 * @dev Hierarchical Diamond Architecture for 50 AMM + 50 Strike Bots
 */

import {LibDiamond} from "./libraries/LibDiamond.sol";
import {IDiamondCut} from "./interfaces/IDiamondCut.sol";
import {IMasterDiamond} from "./interfaces/IMasterDiamond.sol";

contract MasterDiamond is IMasterDiamond {
    using LibDiamond for LibDiamond.DiamondStorage;

    constructor(
        address _contractOwner,
        address _diamondCutFacet
    ) {
        LibDiamond.setContractOwner(_contractOwner);

        // Add the diamondCut external function from the diamondCutFacet
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

    /**
     * @notice Register child diamond (Layer 1)
     */
    function registerChildDiamond(
        ChildDiamondType _type,
        address _childDiamondAddress
    ) external {
        LibDiamond.enforceIsContractOwner();
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        
        require(_childDiamondAddress != address(0), "Invalid address");
        require(ds.childDiamonds[_type] == address(0), "Child already registered");
        
        ds.childDiamonds[_type] = _childDiamondAddress;
        ds.childDiamondTypes.push(_type);
        
        emit ChildDiamondRegistered(_type, _childDiamondAddress);
    }

    /**
     * @notice Execute coordinated operation across all child diamonds
     */
    function executeCoordinatedOperation(
        OperationType _operation,
        bytes memory _data
    ) external returns (bool[] memory successes, uint256[] memory results) {
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        
        require(ds.childDiamonds[ChildDiamondType.LongStrike] != address(0), "Long Strike not registered");
        require(ds.childDiamonds[ChildDiamondType.ShortStrike] != address(0), "Short Strike not registered");
        require(ds.childDiamonds[ChildDiamondType.AMM] != address(0), "AMM not registered");
        
        successes = new bool[](3);
        results = new uint256[](3);
        
        // Execute on Long Strike Diamond
        (bool success1, bytes memory result1) = ds.childDiamonds[ChildDiamondType.LongStrike]
            .call(abi.encodeWithSignature("executeOperation(uint8,bytes)", uint8(_operation), _data));
        successes[0] = success1;
        if (success1 && result1.length > 0) {
            results[0] = abi.decode(result1, (uint256));
        }
        
        // Execute on Short Strike Diamond
        (bool success2, bytes memory result2) = ds.childDiamonds[ChildDiamondType.ShortStrike]
            .call(abi.encodeWithSignature("executeOperation(uint8,bytes)", uint8(_operation), _data));
        successes[1] = success2;
        if (success2 && result2.length > 0) {
            results[1] = abi.decode(result2, (uint256));
        }
        
        // Execute on AMM Diamond
        (bool success3, bytes memory result3) = ds.childDiamonds[ChildDiamondType.AMM]
            .call(abi.encodeWithSignature("executeOperation(uint8,bytes)", uint8(_operation), _data));
        successes[2] = success3;
        if (success3 && result3.length > 0) {
            results[2] = abi.decode(result3, (uint256));
        }
        
        emit CoordinatedOperationExecuted(_operation, successes, results);
    }

    /**
     * @notice Get aggregate statistics from all child diamonds
     */
    function getAggregateStats() external view returns (AggregateStats memory) {
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        
        AggregateStats memory stats;
        
        // Get stats from each child diamond
        if (ds.childDiamonds[ChildDiamondType.LongStrike] != address(0)) {
            (bool success, bytes memory data) = ds.childDiamonds[ChildDiamondType.LongStrike]
                .staticcall(abi.encodeWithSignature("getStats()"));
            if (success) {
                stats.longStrikeStats = abi.decode(data, (StrikeStats));
            }
        }
        
        if (ds.childDiamonds[ChildDiamondType.ShortStrike] != address(0)) {
            (bool success, bytes memory data) = ds.childDiamonds[ChildDiamondType.ShortStrike]
                .staticcall(abi.encodeWithSignature("getStats()"));
            if (success) {
                stats.shortStrikeStats = abi.decode(data, (StrikeStats));
            }
        }
        
        if (ds.childDiamonds[ChildDiamondType.AMM] != address(0)) {
            (bool success, bytes memory data) = ds.childDiamonds[ChildDiamondType.AMM]
                .staticcall(abi.encodeWithSignature("getStats()"));
            if (success) {
                stats.ammStats = abi.decode(data, (AMMStats));
            }
        }
        
        // Calculate totals
        stats.totalCapital = stats.longStrikeStats.totalCapital + 
                           stats.shortStrikeStats.totalCapital + 
                           stats.ammStats.totalCapital;
        stats.totalBots = stats.longStrikeStats.numBots + 
                         stats.shortStrikeStats.numBots + 
                         stats.ammStats.numBots;
        stats.overallWinRate = (stats.longStrikeStats.winRate + 
                               stats.shortStrikeStats.winRate + 
                               stats.ammStats.successRate) / 3;
        
        return stats;
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
