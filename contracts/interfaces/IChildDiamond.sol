// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IChildDiamond {
    enum OperationType {
        Strike,
        Arbitrage,
        Rebalance
    }

    struct StrikeStats {
        uint256 totalCapital;
        uint256 totalStrikes;
        uint256 successfulStrikes;
        uint256 winRate;
        uint8 numBots;
    }

    struct AMMStats {
        uint256 totalCapital;
        uint256 totalArbitrages;
        uint256 successfulArbitrages;
        uint256 successRate;
        uint8 numBots;
        uint8 minConfidence;
    }

    function executeOperation(uint8 _operation, bytes memory _data) external returns (uint256);
    function getStats() external view returns (StrikeStats memory); // For strike diamonds
    function getStats() external view returns (AMMStats memory); // For AMM diamond
}
