// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library LibShortStrike {
    bytes32 constant SHORT_STRIKE_STORAGE_POSITION = keccak256("diamond.short.strike.storage");

    struct ShortStrikeStorage {
        bool isInitialized;
        uint256 initialCapital;
        uint256 totalCapital;
        uint8 numBots;
        uint256 capitalPerBot;
        uint256 totalStrikes;
        uint256 successfulStrikes;
        uint256 winRate;
        mapping(uint8 => uint256) botCapital;
        mapping(uint8 => uint256) botStrikes;
        mapping(uint8 => uint256) botSuccessfulStrikes;
    }

    function shortStrikeStorage() internal pure returns (ShortStrikeStorage storage ds) {
        bytes32 position = SHORT_STRIKE_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }
}
