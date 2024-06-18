// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

interface IYieldBase {
  event Stake(address indexed user, uint256 inputAmount, uint256 stakedAmount);
}
