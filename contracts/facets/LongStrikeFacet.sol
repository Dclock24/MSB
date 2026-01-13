// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title LongStrikeFacet
 * @notice Manages 25 Long Strike Bots
 * @dev Facet for Long Strike Diamond
 */

import {LibDiamond} from "../libraries/LibDiamond.sol";
import {LibLongStrike} from "../libraries/LibLongStrike.sol";
import {ILongStrikeFacet} from "../interfaces/ILongStrikeFacet.sol";

contract LongStrikeFacet is ILongStrikeFacet {
    using LibLongStrike for LibLongStrike.LongStrikeStorage;

    modifier onlyOwner() {
        LibDiamond.enforceIsContractOwner();
        _;
    }

    /**
     * @notice Initialize 25 long strike bots
     */
    function initializeLongStrikeBots(uint256 _initialCapital) external onlyOwner {
        require(_initialCapital > 0, "Capital must be > 0");
        
        LibLongStrike.LongStrikeStorage storage s = LibLongStrike.longStrikeStorage();
        s.initialCapital = _initialCapital;
        s.totalCapital = _initialCapital;
        s.numBots = 25; // Fixed at 25 long bots
        s.capitalPerBot = _initialCapital / 25;
        s.isInitialized = true;
        
        emit LongStrikeBotsInitialized(_initialCapital, 25);
    }

    /**
     * @notice Execute long strike across all 25 bots
     */
    function executeLongStrike(
        StrikeOpportunity memory _opportunity
    ) external returns (bool success, uint256 totalProfit) {
        LibLongStrike.LongStrikeStorage storage s = LibLongStrike.longStrikeStorage();
        require(s.isInitialized, "Bots not initialized");
        require(_opportunity.confidence >= 93, "Confidence too low");
        
        // Only execute LONG positions
        require(_opportunity.direction == Direction.Long, "Must be long position");
        
        uint256 profitPerBot = _opportunity.expectedProfit / 25;
        uint256 totalExecutedProfit = 0;
        uint256 successfulBots = 0;
        
        // Execute on all 25 long bots
        for (uint8 i = 0; i < 25; i++) {
            (bool botSuccess, uint256 botProfit) = _executeBotLongStrike(i, _opportunity, profitPerBot);
            if (botSuccess) {
                successfulBots++;
                totalExecutedProfit += botProfit;
            }
        }
        
        s.totalCapital += totalExecutedProfit;
        s.totalStrikes++;
        s.successfulStrikes += successfulBots;
        s.winRate = (s.successfulStrikes * 100) / (s.totalStrikes * 25);
        
        success = successfulBots >= (25 * 93) / 100; // 93% threshold
        totalProfit = totalExecutedProfit;
        
        emit LongStrikeExecuted(s.totalStrikes, successfulBots, totalExecutedProfit, s.winRate);
    }

    function _executeBotLongStrike(
        uint8 _botId,
        StrikeOpportunity memory _opportunity,
        uint256 _expectedProfit
    ) internal returns (bool success, uint256 profit) {
        LibLongStrike.LongStrikeStorage storage s = LibLongStrike.longStrikeStorage();
        
        if (s.botCapital[_botId] < s.capitalPerBot / 10) {
            return (false, 0);
        }
        
        // 93% success rate for long positions
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

    /**
     * @notice Rebalance capital across long bots
     */
    function rebalanceLongBots() external returns (uint256) {
        LibLongStrike.LongStrikeStorage storage s = LibLongStrike.longStrikeStorage();
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
        
        emit LongBotsRebalanced(totalBotCapital);
        return totalBotCapital;
    }

    /**
     * @notice Get long strike statistics
     */
    function getLongStrikeStats() external view returns (StrikeStats memory) {
        LibLongStrike.LongStrikeStorage storage s = LibLongStrike.longStrikeStorage();
        return StrikeStats({
            totalCapital: s.totalCapital,
            totalStrikes: s.totalStrikes,
            successfulStrikes: s.successfulStrikes,
            winRate: s.winRate,
            numBots: s.numBots
        });
    }
}
