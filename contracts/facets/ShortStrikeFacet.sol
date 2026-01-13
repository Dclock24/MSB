// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ShortStrikeFacet
 * @notice Manages 25 Short Strike Bots
 */

import {LibDiamond} from "../libraries/LibDiamond.sol";
import {LibShortStrike} from "../libraries/LibShortStrike.sol";
import {IShortStrikeFacet} from "../interfaces/IShortStrikeFacet.sol";

contract ShortStrikeFacet is IShortStrikeFacet {
    using LibShortStrike for LibShortStrike.ShortStrikeStorage;

    modifier onlyOwner() {
        LibDiamond.enforceIsContractOwner();
        _;
    }

    function initializeShortStrikeBots(uint256 _initialCapital) external onlyOwner {
        require(_initialCapital > 0, "Capital must be > 0");
        
        LibShortStrike.ShortStrikeStorage storage s = LibShortStrike.shortStrikeStorage();
        s.initialCapital = _initialCapital;
        s.totalCapital = _initialCapital;
        s.numBots = 25; // Fixed at 25 short bots
        s.capitalPerBot = _initialCapital / 25;
        s.isInitialized = true;
        
        emit ShortStrikeBotsInitialized(_initialCapital, 25);
    }

    function executeShortStrike(
        StrikeOpportunity memory _opportunity
    ) external returns (bool success, uint256 totalProfit) {
        LibShortStrike.ShortStrikeStorage storage s = LibShortStrike.shortStrikeStorage();
        require(s.isInitialized, "Bots not initialized");
        require(_opportunity.confidence >= 93, "Confidence too low");
        require(_opportunity.direction == Direction.Short, "Must be short position");
        
        uint256 profitPerBot = _opportunity.expectedProfit / 25;
        uint256 totalExecutedProfit = 0;
        uint256 successfulBots = 0;
        
        for (uint8 i = 0; i < 25; i++) {
            (bool botSuccess, uint256 botProfit) = _executeBotShortStrike(i, _opportunity, profitPerBot);
            if (botSuccess) {
                successfulBots++;
                totalExecutedProfit += botProfit;
            }
        }
        
        s.totalCapital += totalExecutedProfit;
        s.totalStrikes++;
        s.successfulStrikes += successfulBots;
        s.winRate = (s.successfulStrikes * 100) / (s.totalStrikes * 25);
        
        success = successfulBots >= (25 * 93) / 100;
        totalProfit = totalExecutedProfit;
        
        emit ShortStrikeExecuted(s.totalStrikes, successfulBots, totalExecutedProfit, s.winRate);
    }

    function _executeBotShortStrike(
        uint8 _botId,
        StrikeOpportunity memory _opportunity,
        uint256 _expectedProfit
    ) internal returns (bool success, uint256 profit) {
        LibShortStrike.ShortStrikeStorage storage s = LibShortStrike.shortStrikeStorage();
        
        if (s.botCapital[_botId] < s.capitalPerBot / 10) {
            return (false, 0);
        }
        
        if (_opportunity.confidence >= 93) {
            uint256 variation = (_expectedProfit * 5) / 100;
            profit = _expectedProfit + (variation * (block.timestamp % 11 - 5)) / 5;
            success = true;
            s.botCapital[_botId] += profit;
        } else {
            profit = (s.botCapital[_botId] * 2) / 100;
            s.botCapital[_botId] -= profit;
            success = false;
        }
        
        s.botStrikes[_botId]++;
        if (success) {
            s.botSuccessfulStrikes[_botId]++;
        }
    }

    function rebalanceShortBots() external returns (uint256) {
        LibShortStrike.ShortStrikeStorage storage s = LibShortStrike.shortStrikeStorage();
        require(s.isInitialized, "Bots not initialized");
        
        uint256 totalBotCapital = 0;
        for (uint8 i = 0; i < 25; i++) {
            totalBotCapital += s.botCapital[i];
        }
        
        uint256 targetPerBot = totalBotCapital / 25;
        
        for (uint8 i = 0; i < 25; i++) {
            if (s.botCapital[i] < targetPerBot) {
                uint256 needed = targetPerBot - s.botCapital[i];
                if (s.totalCapital >= needed) {
                    s.botCapital[i] += needed;
                    s.totalCapital -= needed;
                }
            }
        }
        
        emit ShortBotsRebalanced(totalBotCapital);
        return totalBotCapital;
    }

    function getShortStrikeStats() external view returns (StrikeStats memory) {
        LibShortStrike.ShortStrikeStorage storage s = LibShortStrike.shortStrikeStorage();
        return StrikeStats({
            totalCapital: s.totalCapital,
            totalStrikes: s.totalStrikes,
            successfulStrikes: s.successfulStrikes,
            winRate: s.winRate,
            numBots: s.numBots
        });
    }
}
