// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IYieldBase.sol';

/// @title Interface for EthYield
interface IEthYield is IYieldBase {
  /// @notice method to start Lido staking + EigenLayer restaking process
  /// @param stakeId unique id of the stake from Warden
  /// @param amount an amount of either weth or native eth to be staked
  /// @dev in the case of native eth a msg.value must strictly equal the amount arg; the tx reverts otherwise
  function stake(uint64 stakeId, uint256 amount) external returns (uint256);

  /// @notice method to start EigenLayer + Lido unstaking process
  /// @param unstakeId unique id of the unstake
  /// @param lpAmount lp amount to be unstaked
  function unstake(uint64 unstakeId, uint256 lpAmount) external;

  /// @notice completes if possible the oldest non-fulfilled withdrawal requests from both EigenLayer and Lido queues
  function reinit() external returns (uint64, uint256);
}
