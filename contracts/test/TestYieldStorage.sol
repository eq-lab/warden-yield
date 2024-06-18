// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import '../YieldStorage.sol';

contract TestYieldStorage is UUPSUpgradeable, Ownable2StepUpgradeable, YieldStorage {
  uint256 constant STAKING_RATIO = 75;
  uint256 constant ONE = 100;

  function initialize() external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function stake(uint256 amount) external {
    uint256 stakedAmount = getStakedAmount(amount);
    _addStake(msg.sender, amount, stakedAmount);
  }

  function getStakedAmount(uint256 amount) public pure returns (uint256) {
    return (amount * STAKING_RATIO) / ONE;
  }
}
