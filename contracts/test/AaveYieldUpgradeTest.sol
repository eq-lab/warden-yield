// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '../AaveYield.sol';

contract AaveYieldUpgradeTest is AaveYield {
  function upgradedTest() external pure returns (bool) {
    return true;
  }
}
