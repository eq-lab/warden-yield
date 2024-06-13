// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol";

import "./interfaces/IYield.sol";
import "./YieldStorage.sol";

abstract contract YieldBase is UUPSUpgradeable, Ownable2StepUpgradeable, YieldStorage, IYield {
  address public token;
  address public underlyingProtocolAddress;

  function initialize(address _token, address _underlyingProtocolAddress) public initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    token = _token;
    underlyingProtocolAddress = _underlyingProtocolAddress;
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function stake(uint256 amount) external returns (uint256 stakedAmount) {
    stakedAmount = _stake(amount);

    StakingData storage $ =  _getStakingDataStorage();
    $._totalInputAmount += amount;
    $._inputAmount[msg.sender] += amount;
    $._stakedAmount[msg.sender] += stakedAmount;
    $._totalStakedAmount += stakedAmount;
  }

  function _stake(uint256 amount) internal virtual returns (uint256);
}
