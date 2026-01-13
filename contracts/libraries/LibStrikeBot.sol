// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library LibStrikeBot {
    bytes32 constant STRIKE_BOT_STORAGE_POSITION = keccak256("diamond.strike.bot.storage");

    struct StrikeBotStorage {
        bool isInitialized;
        uint256 initialCapital;
        uint256 totalCapital;
        uint8 numBots;
        uint256 capitalPerBot;
        uint256 totalStrikes;
        uint256 successfulStrikes;
        uint256 winRate; // Percentage (0-100)
        mapping(uint8 => uint256) botCapital;
        mapping(uint8 => uint256) botStrikes;
        mapping(uint8 => uint256) botSuccessfulStrikes;
        mapping(address => bool) authorizedOperators;
    }

    function strikeBotStorage() internal pure returns (StrikeBotStorage storage ds) {
        bytes32 position = STRIKE_BOT_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }

    function enforceIsAuthorized() internal view {
        StrikeBotStorage storage s = strikeBotStorage();
        require(
            s.authorizedOperators[msg.sender] || 
            LibDiamond.contractOwner() == msg.sender,
            "Not authorized"
        );
    }
}

// Import LibDiamond
import {LibDiamond} from "./LibDiamond.sol";
