// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library LibAMM {
    bytes32 constant AMM_STORAGE_POSITION = keccak256("diamond.amm.storage");

    struct AMMStorage {
        bool isInitialized;
        uint256 initialCapital;
        uint256 totalCapital;
        uint8 numBots;
        uint256 capitalPerBot;
        uint256 totalArbitrages;
        uint256 successfulArbitrages;
        uint256 successRate;
        uint8 minConfidence;
        mapping(address => bool) registeredPools;
        address[] poolList;
        mapping(uint8 => uint256) botCapital;
        mapping(uint8 => uint256) botArbitrages;
        mapping(uint8 => uint256) botSuccessfulArbitrages;
    }

    function ammStorage() internal pure returns (AMMStorage storage ds) {
        bytes32 position = AMM_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }
}
