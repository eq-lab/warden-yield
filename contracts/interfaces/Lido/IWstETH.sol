// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/IERC20.sol';

/// @title Interface for WstETH
interface IWstETH is IERC20 {
  function wrap(uint256 _stETHAmount) external returns (uint256);
  function unwrap(uint256 _wstETHAmount) external returns (uint256);
}