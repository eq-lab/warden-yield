// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/IERC20.sol';

/// @title Interface for StETH
interface IStETH is IERC20 {
  function submit(address referral) external payable returns (uint256);

  function getTotalShares() external view returns (uint256);

  function getTotalPooledEther() external view returns (uint256);
}
