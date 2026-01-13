// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IAMMFacet {
    struct ArbitrageRequest {
        uint8 confidence;
        uint256 amountIn;
        address tokenIn;
        address tokenOut;
        address poolA;
        address poolB;
        uint256 expectedProfit;
        uint256 minProfit;
        uint256 gasEstimate;
    }

    struct AMMStats {
        uint256 totalCapital;
        uint256 totalArbitrages;
        uint256 successfulArbitrages;
        uint256 successRate;
        uint8 numBots;
        uint8 minConfidence;
    }

    event AMMBotsInitialized(uint256 initialCapital, uint8 numBots, uint256 numPools);
    event AMMArbitrageExecuted(
        uint256 totalArbitrages,
        uint256 successfulBots,
        uint256 totalProfit,
        uint256 successRate
    );
    event AMMBotsRebalanced(uint256 totalCapital);

    function initializeAMMBots(uint256 _initialCapital, address[] memory _dexPools) external;
    function executeAMMArbitrage(ArbitrageRequest memory _request) 
        external returns (bool success, uint256 totalProfit);
    function rebalanceAMMBots() external returns (uint256);
    function getAMMStats() external view returns (AMMStats memory);
}
