// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IShortStrikeFacet {
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

    event ShortStrikeBotsInitialized(uint256 initialCapital, uint8 numBots);
    event ShortStrikeExecuted(
        uint256 totalStrikes,
        uint256 successfulBots,
        uint256 totalProfit,
        uint256 winRate
    );
    event ShortBotsRebalanced(uint256 totalCapital);

    function initializeShortStrikeBots(uint256 _initialCapital) external;
    function executeShortStrike(StrikeOpportunity memory _opportunity) 
        external returns (bool success, uint256 totalProfit);
    function rebalanceShortBots() external returns (uint256);
    function getShortStrikeStats() external view returns (StrikeStats memory);
}
