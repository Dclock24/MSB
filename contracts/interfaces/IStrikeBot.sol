// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IStrikeBot {
    struct StrikeOpportunity {
        uint8 confidence; // 0-100 (93 = 93%)
        uint256 expectedProfit;
        address tokenPair;
        uint256 entryPrice;
        uint256 targetPrice;
        uint256 stopLoss;
    }

    event StrikeBotsInitialized(uint256 initialCapital, uint8 numBots);
    event CoordinatedStrikeExecuted(
        uint256 totalStrikes,
        uint256 successfulBots,
        uint8 totalBots,
        uint256 totalProfit,
        uint256 winRate
    );
    event CapitalRebalanced(uint256 totalCapital, uint8 numBots);

    function initializeStrikeBots(uint256 _initialCapital, uint8 _numBots) external;
    function executeCoordinatedStrike(StrikeOpportunity memory _opportunity) 
        external returns (bool success, uint256 totalProfit);
    function getStrikeBotStats() external view returns (
        uint256 totalCapital,
        uint256 totalStrikes,
        uint256 successfulStrikes,
        uint256 winRate,
        uint8 numBots,
        uint256 capitalPerBot
    );
    function getBotStatus(uint8 _botId) external view returns (
        uint256 capital,
        uint256 strikesExecuted,
        uint256 successfulStrikes,
        bool isActive
    );
    function rebalanceCapital() external;
}
