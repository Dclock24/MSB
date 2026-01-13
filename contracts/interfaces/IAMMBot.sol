// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IAMMBot {
    struct Prediction {
        uint8 confidence; // 0-100 (93 = 93%)
        uint256 amountIn;
        address tokenIn;
        address tokenOut;
    }

    struct ArbitragePath {
        address poolA;
        address poolB;
        uint256 priceA;
        uint256 priceB;
        uint256 minProfit;
        uint256 gasEstimate;
    }

    event AMMBotsInitialized(uint8 minConfidence, uint256 numPools);
    event ArbitrageExecuted(
        uint256 totalArbitrages,
        bool success,
        uint256 profit,
        uint8 confidence,
        uint256 successRate
    );
    event PoolRegistered(address pool);
    event MinConfidenceUpdated(uint8 minConfidence);

    function initializeAMMBots(uint8 _minConfidence, address[] memory _dexPools) external;
    function executePredictiveArbitrage(
        Prediction memory _prediction,
        ArbitragePath memory _arbitragePath
    ) external returns (bool success, uint256 profit);
    function getAMMBotStats() external view returns (
        uint256 totalCapital,
        uint256 totalArbitrages,
        uint256 successfulArbitrages,
        uint256 successRate,
        uint256 totalProfit,
        uint8 minConfidence
    );
    function registerPool(address _pool) external;
    function setMinConfidence(uint8 _minConfidence) external;
}
