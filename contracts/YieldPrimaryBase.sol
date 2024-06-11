// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol";

import "./interfaces/IYieldPrimaryBase.sol";

abstract contract YieldPrimaryBase is UUPSUpgradeable, Ownable2StepUpgradeable, IYieldPrimaryBase {
  uint256 public totalAmount;
  address public protocolAddress;

  function initialize(address _protocolAddress) public initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    protocolAddress = _protocolAddress;
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function stake(uint256 amount) external returns (uint256 stakedAmount) {
    stakedAmount = _stake(amount);
    totalAmount += stakedAmount;
  }

  function _stake(uint256 amount) internal virtual returns (uint256);
}
