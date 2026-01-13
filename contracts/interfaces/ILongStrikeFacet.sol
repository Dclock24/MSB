// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface ILongStrikeFacet {
    enum Direction {
        Long,
        Short
    }

    struct StrikeOpportunity {
        uint8 confidence;
        Direction direction;
        uint256 expectedProfit;
        address tokenPair;
        uint256 entryPrice;
        uint256 targetPrice;
        uint256 stopLoss;
    }

    struct StrikeStats {
        uint256 totalCapital;
        uint256 totalStrikes;
        uint256 successfulStrikes;
        uint256 winRate;
        uint8 numBots;
    }

    event LongStrikeBotsInitialized(uint256 initialCapital, uint8 numBots);
    event LongStrikeExecuted(
        uint256 totalStrikes,
        uint256 successfulBots,
        uint256 totalProfit,
        uint256 winRate
    );
    event LongBotsRebalanced(uint256 totalCapital);

    function initializeLongStrikeBots(uint256 _initialCapital) external;
    function executeLongStrike(StrikeOpportunity memory _opportunity) 
        external returns (bool success, uint256 totalProfit);
    function rebalanceLongBots() external returns (uint256);
    function getLongStrikeStats() external view returns (StrikeStats memory);
}
