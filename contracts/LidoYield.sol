// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './interactors/EigenLayerInteractor.sol';
import './interactors/LidoInteractor.sol';
import './interfaces/ILidoYield.sol';
import './YieldStorage.sol';

contract LidoYield is
  UUPSUpgradeable,
  Ownable2StepUpgradeable,
  EigenLayerInteractor,
  LidoInteractor,
  YieldStorage,
  ILidoYield
{
  function initialize(address stETH, address wETH9, address elStrategy, address elStrategyManager) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __EigenLayerInteractor_init(stETH, elStrategy, elStrategyManager);
    __LidoInteractor_init(stETH, wETH9);
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function stake(uint256 amount) external payable returns (uint256 eigenLayerShares) {
    uint256 lidoShares = _lidoStake(amount);
    eigenLayerShares = _eigenLayerRestake(lidoShares);
    _addStake(msg.sender, amount, eigenLayerShares);
    emit Stake(msg.sender, amount, eigenLayerShares);
  }
}
