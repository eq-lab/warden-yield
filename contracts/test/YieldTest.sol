// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../YieldBase.sol';

contract YieldTest is YieldBase {
  function _stake(uint256 amount) internal pure override returns (uint256) {
    return amount;
  }
}
