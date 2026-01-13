// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IMasterDiamond {
    enum ChildDiamondType {
        LongStrike,
        ShortStrike,
        AMM
    }

    enum OperationType {
        Strike,
        Arbitrage,
        Rebalance
    }

    struct StrikeStats {
        uint256 totalCapital;
        uint256 totalStrikes;
        uint256 successfulStrikes;
        uint256 winRate; // Percentage
        uint8 numBots;
    }

    struct AMMStats {
        uint256 totalCapital;
        uint256 totalArbitrages;
        uint256 successfulArbitrages;
        uint256 successRate; // Percentage
        uint8 numBots;
        uint8 minConfidence;
    }

    struct AggregateStats {
        StrikeStats longStrikeStats;
        StrikeStats shortStrikeStats;
        AMMStats ammStats;
        uint256 totalCapital;
        uint8 totalBots;
        uint256 overallWinRate;
    }

    event ChildDiamondRegistered(ChildDiamondType indexed diamondType, address indexed diamondAddress);
    event CoordinatedOperationExecuted(
        OperationType indexed operation,
        bool[] successes,
        uint256[] results
    );

    function registerChildDiamond(ChildDiamondType _type, address _childDiamondAddress) external;
    function executeCoordinatedOperation(
        OperationType _operation,
        bytes memory _data
    ) external returns (bool[] memory successes, uint256[] memory results);
    function getAggregateStats() external view returns (AggregateStats memory);
}
