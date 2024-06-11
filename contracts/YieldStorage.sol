// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol";

import "./interfaces/IYieldPrimaryBase.sol";

contract YieldStorage is UUPSUpgradeable, Ownable2StepUpgradeable {
  using SafeERC20 for IERC20;

  address private primaryContract;
  address public token;
  mapping (address => uint256) public getInputAmount;
  mapping (address => uint256) public getStakedAmount;
  mapping (address => string) public getWardenAddress;

  event Stake(uint256 amount, uint256 stakedAmount);
  event WardenAddressSet(address indexed user, string wardenAddress);

  function initialize(address _primaryContract, address _token) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    primaryContract = _primaryContract;
    token = _token;
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function stake(uint256 amount) external returns (uint256 stakedAmount) {
    address _primaryContract = primaryContract;
    IERC20(token).transferFrom(msg.sender, _primaryContract, amount);
    stakedAmount = IYieldPrimaryBase(_primaryContract).stake(amount);

    getInputAmount[msg.sender] += amount;
    getStakedAmount[msg.sender] += stakedAmount;

    emit Stake(amount, stakedAmount);
  }

  function setWardenAddress(string calldata wardenAddress) external {
    require(verifyWardenAddress(wardenAddress), "Wrong input address format");
    getWardenAddress[msg.sender] = wardenAddress;
    emit WardenAddressSet(msg.sender, wardenAddress);
  }

  function verifyWardenAddress(string calldata /*wardenAddress*/) pure public returns (bool) {
    return true;
  }
}
