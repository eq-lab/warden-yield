// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './libraries/Errors.sol';
import './interactors/AaveInteractor.sol';
import './YieldStorage.sol';
import './interfaces/IAaveYield.sol';

contract AaveYieldV2 is UUPSUpgradeable, Ownable2StepUpgradeable, AaveInteractor, YieldStorage, IYieldBase {
  /// @dev method called during the contract upgrade
  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function withdraw(address token) external {
    uint256 userShares = userShares(msg.sender, token);
    uint256 withdrawAmount = getUserUnderlyingAmount(msg.sender, token);

    address aavePool = getAavePool();
    address aToken = IPool(aavePool).getReserveData(token).aTokenAddress;
    uint256 poolBalance = IERC20(aToken).balanceOf(address(this));
    if (poolBalance < withdrawAmount) {
      withdrawAmount = poolBalance;
    }

    if (withdrawAmount == 0) revert Errors.ZeroAmount();
    _removeStake(msg.sender, token);
    _aaveWithdraw(token, withdrawAmount);

    emit Withdraw(msg.sender, token, withdrawAmount);
  }

  /// @dev returns current user balance available to withdraw
  /// @param user user address
  /// @param underlyingToken underlying token address
  /// @return availableToWithdraw available to withdraw amount
  function getUserUnderlyingAmount(
    address user,
    address underlyingToken
  ) public view returns (uint256 availableToWithdraw) {
    uint256 scaledDeposit = userShares(user, underlyingToken);
    availableToWithdraw = _getBalanceFromScaled(scaledDeposit, underlyingToken);
  }
}
