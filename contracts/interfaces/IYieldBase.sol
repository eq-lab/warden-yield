// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

/// @title Interface with general events for all Yield contracts
interface IYieldBase {
  /// @notice emitted after the 'stake' call occured
  /// @param user user who initiated 'stake' call
  /// @param token token used in 'stake' call
  /// @param stakedAmount amount of 'token' staked
  /// @param shares amount of shares received during 'stake' call
  event Stake(address indexed user, address indexed token, uint256 stakedAmount, uint256 shares);

  /// @notice emitted after the 'withdraw' call occured
  /// @param user user who initiated 'withdraw' call
  /// @param token token getting unlocked
  /// @param withdrawAmount amount of 'token' withdrawn
  event Withdraw(address indexed user, address indexed token, uint256 withdrawAmount);

  /// @notice emitted after the 'withdraws' were enabled
  event EnableWithdrawals();

  /// @notice emitted after the 'withdraws' were disabled
  event DisableWithdrawals();
}
