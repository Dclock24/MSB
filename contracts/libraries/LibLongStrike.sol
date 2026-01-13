// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library LibLongStrike {
    bytes32 constant LONG_STRIKE_STORAGE_POSITION = keccak256("diamond.long.strike.storage");

    struct LongStrikeStorage {
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

    function longStrikeStorage() internal pure returns (LongStrikeStorage storage ds) {
        bytes32 position = LONG_STRIKE_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }
}
