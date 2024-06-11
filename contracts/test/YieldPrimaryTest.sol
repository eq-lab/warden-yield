// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import "../YieldPrimaryBase.sol";

contract YieldPrimaryTest is YieldPrimaryBase {
  function _stake(uint256 amount) internal pure override returns (uint256) {
    return amount;
  }
}
