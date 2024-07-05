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

  /// @notice returns current user balance in Aave pool
  function getUserUnderlyingAmount(
    address user,
    address underlyingToken
  ) external view returns (uint256 availableToWithdraw);
}
