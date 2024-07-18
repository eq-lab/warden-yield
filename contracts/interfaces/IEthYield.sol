// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IYieldBase.sol';

/// @title Interface for EthYield
interface IEthYield is IYieldBase {
  /// @notice method to start Lido staking + EigenLayer restaking process
  /// @param amount an amount of either weth or native eth to be staked
  /// @dev in the case of native eth a msg.value must strictly equal the amount arg; the tx reverts otherwise
  function stake(uint256 amount) external returns (uint256);

  /// @notice method to start EigenLayer + Lido unstaking process
  /// @param unstakeId unique id of the unstake
  /// @param sharesAmount EigenLayer shares to withdraw and received stEth to convert to Eth
  function unstake(uint64 unstakeId, uint256 sharesAmount) external;

  /// @notice completes if possible the oldest non-fulfilled withdrawal requests from both EigenLayer and Lido queues
  function reinit() external returns (uint64, uint256);
}
