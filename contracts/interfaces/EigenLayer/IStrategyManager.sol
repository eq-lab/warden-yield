// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IStrategy.sol';

/// @dev https://github.com/Layr-Labs/eigenlayer-contracts/blob/dev/src/contracts/interfaces/IStrategyManager.sol
interface IStrategyManager {
  function depositIntoStrategy(IStrategy strategy, IERC20 token, uint256 amount) external returns (uint256 shares);

  /// @notice Used by the DelegationManager to remove a Staker's shares from a particular strategy when entering the withdrawal queue
  function removeShares(address staker, IStrategy strategy, uint256 shares) external;

  /// @notice Used by the DelegationManager to award a Staker some shares that have passed through the withdrawal queue
  function addShares(address staker, IERC20 token, IStrategy strategy, uint256 shares) external;

  /// @notice Used by the DelegationManager to convert withdrawn shares to tokens and send them to a recipient
  function withdrawSharesAsTokens(address recipient, IStrategy strategy, uint256 shares, IERC20 token) external;

  /// @notice Returns the current shares of `user` in `strategy`
  function stakerStrategyShares(address user, IStrategy strategy) external view returns (uint256 shares);

  /**
   * @notice Get all details on the staker's deposits and corresponding shares
   * @return (staker's strategies, shares in these strategies)
   */
  function getDeposits(address staker) external view returns (IStrategy[] memory, uint256[] memory);
}
