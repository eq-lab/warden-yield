// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './interactors/LidoInteractor.sol';
import './interfaces/IYield.sol';
import './YieldStorage.sol';

contract LidoYield is UUPSUpgradeable, Ownable2StepUpgradeable, LidoInteractor, YieldStorage {
  function initialize(address stETH, address wstETH, address wETH9) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __LidoInteractor_init(stETH, wstETH, wETH9);
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function stake(uint256 amount) external payable returns (uint256 stakedAmount) {
    stakedAmount = _stake(amount);
    _addStake(msg.sender, amount, stakedAmount);
    // emit Stake(msg.sender, amount, stakedAmount);
  }
}
