// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './libraries/Errors.sol';
import './interactors/AaveInteractor.sol';
import './YieldStorage.sol';
import './interfaces/IAaveYield.sol';

contract AaveYield is UUPSUpgradeable, Ownable2StepUpgradeable, AaveInteractor, YieldStorage, IAaveYield {
  /// @notice initialize function used during contract deployment
  /// @param aavePool address of a Aave pool
  /// @param tokens array with addresses of tokens which will be used in the Aave pool
  function initialize(address aavePool, address[] calldata tokens) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __AaveInteractor_init(aavePool, tokens);
  }

  /// @dev method called during the contract upgrade
  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  /// @inheritdoc IAaveYield
  function stake(address token, uint256 amount, string calldata userWardenAddress) external returns (uint256 shares) {
    shares = _aaveStake(token, amount);
    _addStake(msg.sender, token, amount, shares);
    _addWardenAddress(msg.sender, userWardenAddress);

    emit Stake(msg.sender, token, amount, shares);
  }

  /// @inheritdoc IAaveYield
  function getAvailableToWithdraw(address user, address token) public view returns (uint256 availableToWithdraw) {
    uint256 scaledDeposit = userShares(user, token);
    availableToWithdraw = _getBalanceFromScaled(scaledDeposit, token);
  }
}
