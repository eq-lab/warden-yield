// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IYieldBase.sol';

/// @title Interface for EthYield
interface IEthYield is IYieldBase {
  /// @notice method to start Lido staking + EigenLayer restaking process
  /// @param amount an amount of either weth or native eth to be staked
  /// @dev in the case of native eth a msg.value must strictly equal the amount arg; the tx reverts otherwise
  /// @param userWardenAddress warden address of user
  /// @dev if user calls the stake method for the second or more time
  ///      then the same address must be passed as during the first call
  function stake(uint256 amount, string calldata userWardenAddress) external payable returns (uint256);
}
