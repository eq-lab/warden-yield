// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import '../YieldStorage.sol';

contract TestYieldStorage is UUPSUpgradeable, Ownable2StepUpgradeable, YieldStorage {
  uint256 constant STAKING_RATIO = 75;
  uint256 constant LPT_RATIO = 50;
  uint256 constant ONE = 100;
  address WETH9;

  function initialize(address weth9) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    WETH9 = weth9;
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function stake(uint256 amount) external {
    uint256 shares = getStakedAmount(amount);
    uint256 lptAmount = getLptAmount(shares);
    _addStake(shares, lptAmount);
  }

  function getStakedAmount(uint256 amount) public pure returns (uint256) {
    return (amount * STAKING_RATIO) / ONE;
  }

  function getLptAmount(uint256 shares) public pure returns (uint256) {
    return (shares * LPT_RATIO) / ONE;
  }
}
