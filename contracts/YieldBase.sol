// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity =0.8.26;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol";


contract YieldBase is Initializable, UUPSUpgradeable, Ownable2StepUpgradeable {
  function initialize() public initializer {
    __Ownable_init();
    __UUPSUpgradeable_init();
  }

  function stake(uint256 amount, string calldata wardenAddress) external {
    _stake(amount, wardenAddress);
  }
}
