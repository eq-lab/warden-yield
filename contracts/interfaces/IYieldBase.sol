// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

/// @title Interface with general events for all Yield contracts
interface IYieldBase {
  /// @notice emitted after the 'stake' call occured
  /// @param stakeId stake identifier from Warden chain
  /// @param token token used in 'stake' call
  /// @param stakedAmount amount of 'token' staked
  /// @param lpAmount amount of lp token
  event Stake(uint64 indexed stakeId, address indexed token, uint256 stakedAmount, uint256 lpAmount);

  /// @notice emitted after the 'withdraw' call occured
  /// @param unstakeId unstake identifier from Warden chain
  /// @param token token getting unlocked
  /// @param unstakedAmount amount of 'token' withdrawn
  event Unstake(uint64 indexed unstakeId, address indexed token, uint256 unstakedAmount);
}
