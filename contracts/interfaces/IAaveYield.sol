// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IYieldBase.sol';

/// @title Interface for AaveYield
interface IAaveYield is IYieldBase {
  /// @notice method to supply tokens to Aave pool
  /// @param token address of a token which will be supplied to Aave pool
  /// @param amount an amount of either weth or native eth to be staked
  /// @dev in the case of native eth a msg.value must strictly equal the amount arg; the tx reverts otherwise
  /// @param userWardenAddress warden address of user
  /// @dev if user calls the stake method for the second or more time
  ///      then the same address must be passed as during the first call
  function stake(address token, uint256 amount, string calldata userWardenAddress) external returns (uint256);

  /// @notice method to withdraw token from Aave pool
  /// @dev in the first version withdraws are disabled by default
  /// @param token address of a token to be withdrawn
  function withdraw(address token) external returns (uint256 withdrawAmount);

  /// @notice returns current user balance in Aave pool
  function getAvailableToWithdraw(address user, address token) external view returns (uint256 availableToWithdraw);
}
