// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library LibAMMBot {
    bytes32 constant AMM_BOT_STORAGE_POSITION = keccak256("diamond.amm.bot.storage");

    struct AMMBotStorage {
        bool isInitialized;
        uint256 totalCapital;
        uint256 totalArbitrages;
        uint256 successfulArbitrages;
        uint256 successRate; // Percentage (0-100)
        uint256 totalProfit;
        uint8 minConfidence; // Minimum confidence threshold (93 = 93%)
        mapping(address => bool) registeredPools;
        address[] poolList;
        mapping(address => bool) authorizedOperators;
    }

    function ammBotStorage() internal pure returns (AMMBotStorage storage ds) {
        bytes32 position = AMM_BOT_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }

    function enforceIsAuthorized() internal view {
        AMMBotStorage storage s = ammBotStorage();
        require(
            s.authorizedOperators[msg.sender] || 
            LibDiamond.contractOwner() == msg.sender,
            "Not authorized"
        );
    }
}

// Import LibDiamond
import {LibDiamond} from "./LibDiamond.sol";
