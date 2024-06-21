// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IStrategy.sol';

/// @dev https://github.com/Layr-Labs/eigenlayer-contracts/blob/dev/src/contracts/interfaces/IStrategyManager.sol
interface IStrategyManager {
  function depositIntoStrategy(IStrategy strategy, IERC20 token, uint256 amount) external returns (uint256 shares);

  function removeShares(address staker, IStrategy strategy, uint256 shares) external;

  function addShares(address staker, IERC20 token, IStrategy strategy, uint256 shares) external;

  function withdrawSharesAsTokens(address recipient, IStrategy strategy, uint256 shares, IERC20 token) external;

  function stakerStrategyShares(address user, IStrategy strategy) external view returns (uint256 shares);

  function getDeposits(address staker) external view returns (IStrategy[] memory, uint256[] memory);

  function strategyIsWhitelistedForDeposit(IStrategy strategy) external view returns (bool);
}
