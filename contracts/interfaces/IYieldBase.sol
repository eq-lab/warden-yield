// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

interface IYieldBase {
  event Stake(address indexed user, address indexed token, uint256 stakedAmount, uint256 shares);
  event Withdraw(address indexed user, address indexed token, uint256 withdrawAmount);
  event EnableWithdrawals();
  event DisableWithdrawals();
}
