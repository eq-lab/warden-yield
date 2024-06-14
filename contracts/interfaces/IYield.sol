// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

interface IYield {
  event Stake(address indexed user, uint256 inputAmount, uint256 stakedAmount);

  function stake(uint256 amount) external returns (uint256);
}
