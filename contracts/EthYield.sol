// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './interactors/EigenLayerInteractor.sol';
import './interactors/LidoInteractor.sol';
import './interfaces/IEthYield.sol';
import './YieldStorage.sol';

contract EthYield is
  UUPSUpgradeable,
  Ownable2StepUpgradeable,
  EigenLayerInteractor,
  LidoInteractor,
  YieldStorage,
  IEthYield
{
  /// @notice initialize function used during contract deployment
  /// @param stETH address of a Lido StETH token
  /// @param wETH9 address of a wrapped ETH
  /// @param elStrategy address of an EigenLayer strategy (an StEth one specifically in this case)
  /// @param elStrategyManager address of an EigenLayer strategy manager
  /// @param elDelegationManager address of an EigenLayer delegation manager
  /// @param elOperator address of an EigenLayer operator to whom all the restaked stEth will be delegated
  /// @dev elOperator MUST NOT require any signature, otherwise the initialize tx will revert
  function initialize(
    address stETH,
    address wETH9,
    address elStrategy,
    address elStrategyManager,
    address elDelegationManager,
    address elOperator
  ) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __EigenLayerInteractor_init(stETH, elStrategy, elStrategyManager, elDelegationManager, elOperator);
    __LidoInteractor_init(stETH, wETH9);
  }

  /// @dev method called during the contract upgrade
  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  /// @inheritdoc IEthYield
  function stake(
    uint256 amount,
    string calldata userWardenAddress
  ) external payable returns (uint256 eigenLayerShares) {
    uint256 stEthAmount = _lidoStake(amount);
    eigenLayerShares = _eigenLayerRestake(stEthAmount);
    address weth = getWeth();
    _addStake(msg.sender, weth, amount, eigenLayerShares);
    _addWardenAddress(msg.sender, userWardenAddress);
    emit Stake(msg.sender, weth, amount, eigenLayerShares);
  }
}
