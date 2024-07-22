// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IYieldBase.sol';

/// @title Interface for AaveYield
interface IAaveYield is IYieldBase {
  /// @notice method to supply tokens to Aave pool
  /// @param amount an amount of either weth or native eth to be staked
  function stake(uint256 amount) external returns (uint256);

  /// @notice returns current user balance in Aave pool
  function getUserUnderlyingAmount(
    address user,
    address underlyingToken
  ) external view returns (uint256 availableToWithdraw);
}
