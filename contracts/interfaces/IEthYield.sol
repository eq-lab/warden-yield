// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IYieldBase.sol';

interface IEthYield is IYieldBase {
  function stake(uint256 amount) external payable returns (uint256);
}
