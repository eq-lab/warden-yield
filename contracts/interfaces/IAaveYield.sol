// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IYieldBase.sol';

interface IAaveYield is IYieldBase {
  function stake(address token, uint256 amount) external returns (uint256);
  function withdraw(address token) external returns (uint256 withdrawAmount);
  function getAvailableToWithdraw(address user, address token) external view returns (uint256 availableToWithdraw);
}
