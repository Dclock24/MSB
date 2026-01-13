// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title StrikeBotFacet
 * @notice Manages all strike bot operations through Diamond Facet
 * @dev Handles 25 parallel strike bots with coordinated execution
 */

import {LibDiamond} from "../libraries/LibDiamond.sol";
import {LibStrikeBot} from "../libraries/LibStrikeBot.sol";
import {IStrikeBot} from "../interfaces/IStrikeBot.sol";

contract StrikeBotFacet is IStrikeBot {
    using LibStrikeBot for LibStrikeBot.StrikeBotStorage;

    modifier onlyOwner() {
        LibDiamond.enforceIsContractOwner();
        _;
    }

    modifier onlyAuthorized() {
        LibStrikeBot.enforceIsAuthorized();
        _;
    }

    /**
     * @notice Initialize strike bot system
     * @param _initialCapital Initial capital in wei
     * @param _numBots Number of bots to deploy (max 25)
     */
    function initializeStrikeBots(
        uint256 _initialCapital,
        uint8 _numBots
    ) external onlyOwner {
        require(_numBots > 0 && _numBots <= 25, "Invalid bot count");
        require(_initialCapital > 0, "Capital must be > 0");
        
        LibStrikeBot.StrikeBotStorage storage s = LibStrikeBot.strikeBotStorage();
        s.initialCapital = _initialCapital;
        s.totalCapital = _initialCapital;
        s.numBots = _numBots;
        s.capitalPerBot = _initialCapital / _numBots;
        s.isInitialized = true;
        
        emit StrikeBotsInitialized(_initialCapital, _numBots);
    }

    /**
     * @notice Execute coordinated strike across all bots
     * @param _opportunity Strike opportunity data
     * @return success Whether strike was successful
     * @return totalProfit Total profit from strike
     */
    function executeCoordinatedStrike(
        StrikeOpportunity memory _opportunity
    ) external onlyAuthorized returns (bool success, uint256 totalProfit) {
        LibStrikeBot.StrikeBotStorage storage s = LibStrikeBot.strikeBotStorage();
        require(s.isInitialized, "Bots not initialized");
        require(s.numBots > 0, "No bots available");
        
        // Validate opportunity
        require(_opportunity.confidence >= 93, "Confidence too low"); // 93% minimum
        require(_opportunity.expectedProfit > 0, "No profit expected");
        
        // Distribute strike across bots
        uint256 profitPerBot = _opportunity.expectedProfit / s.numBots;
        uint256 totalExecutedProfit = 0;
        uint256 successfulBots = 0;
        
        // Execute strike on each bot
        for (uint8 i = 0; i < s.numBots; i++) {
            (bool botSuccess, uint256 botProfit) = _executeBotStrike(i, _opportunity, profitPerBot);
            if (botSuccess) {
                successfulBots++;
                totalExecutedProfit += botProfit;
            }
        }
        
        // Update capital
        s.totalCapital += totalExecutedProfit;
        s.totalStrikes++;
        s.successfulStrikes += successfulBots;
        
        // Calculate success rate
        s.winRate = (s.successfulStrikes * 100) / (s.totalStrikes * s.numBots);
        
        success = successfulBots >= (s.numBots * 93) / 100; // 93% success threshold
        totalProfit = totalExecutedProfit;
        
        emit CoordinatedStrikeExecuted(
            s.totalStrikes,
            successfulBots,
            s.numBots,
            totalExecutedProfit,
            s.winRate
        );
    }

    /**
     * @notice Execute strike on individual bot
     */
    function _executeBotStrike(
        uint8 _botId,
        StrikeOpportunity memory _opportunity,
        uint256 _expectedProfit
    ) internal returns (bool success, uint256 profit) {
        LibStrikeBot.StrikeBotStorage storage s = LibStrikeBot.strikeBotStorage();
        
        // Check bot has sufficient capital
        if (s.botCapital[_botId] < s.capitalPerBot / 10) {
            return (false, 0);
        }
        
        // Simulate execution (in production, call actual DEX)
        // 93% success rate based on confidence
        if (_opportunity.confidence >= 93) {
            // Apply small slippage/variation
            uint256 variation = (_expectedProfit * 5) / 100; // ±5%
            profit = _expectedProfit + (variation * (block.timestamp % 11 - 5)) / 5;
            success = true;
            
            // Update bot capital
            s.botCapital[_botId] += profit;
        } else {
            // Small loss on failure
            profit = (s.botCapital[_botId] * 2) / 100; // 2% loss
            s.botCapital[_botId] -= profit;
            success = false;
        }
    }

    /**
     * @notice Get strike bot statistics
     */
    function getStrikeBotStats() external view returns (
        uint256 totalCapital,
        uint256 totalStrikes,
        uint256 successfulStrikes,
        uint256 winRate,
        uint8 numBots,
        uint256 capitalPerBot
    ) {
        LibStrikeBot.StrikeBotStorage storage s = LibStrikeBot.strikeBotStorage();
        return (
            s.totalCapital,
            s.totalStrikes,
            s.successfulStrikes,
            s.winRate,
            s.numBots,
            s.capitalPerBot
        );
    }

    /**
     * @notice Get individual bot status
     */
    function getBotStatus(uint8 _botId) external view returns (
        uint256 capital,
        uint256 strikesExecuted,
        uint256 successfulStrikes,
        bool isActive
    ) {
        LibStrikeBot.StrikeBotStorage storage s = LibStrikeBot.strikeBotStorage();
        require(_botId < s.numBots, "Invalid bot ID");
        
        return (
            s.botCapital[_botId],
            s.botStrikes[_botId],
            s.botSuccessfulStrikes[_botId],
            s.botCapital[_botId] > 0
        );
    }

    /**
     * @notice Rebalance capital across bots
     */
    function rebalanceCapital() external onlyAuthorized {
        LibStrikeBot.StrikeBotStorage storage s = LibStrikeBot.strikeBotStorage();
        require(s.isInitialized, "Bots not initialized");
        
        uint256 totalBotCapital = 0;
        for (uint8 i = 0; i < s.numBots; i++) {
            totalBotCapital += s.botCapital[i];
        }
        
        uint256 targetPerBot = totalBotCapital / s.numBots;
        
        // Redistribute to balance
        for (uint8 i = 0; i < s.numBots; i++) {
            if (s.botCapital[i] < targetPerBot) {
                uint256 needed = targetPerBot - s.botCapital[i];
                if (s.totalCapital >= needed) {
                    s.botCapital[i] += needed;
                    s.totalCapital -= needed;
                }
            }
        }
        
        emit CapitalRebalanced(totalBotCapital, s.numBots);
    }
}
