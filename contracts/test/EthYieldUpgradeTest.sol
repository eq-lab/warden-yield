// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '../EthYield.sol';

contract EthYieldUpgradeTest is EthYield {
  function upgradedTest() external pure returns (bool) {
    return true;
  }
}
