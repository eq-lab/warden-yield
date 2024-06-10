// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

contract VaultV2 is Initializable, UUPSUpgradeable, Ownable2StepUpgradeable {
  using SafeERC20 for IERC20;

  address public token;
  mapping(address => uint256) public balance;

  error ZeroAmount();
  error InsufficientAmount(uint256 balance, uint256 requested);

  function initialize() public initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function deposit(uint256 amount) external {
    if (amount == 0) revert ZeroAmount();
    balance[msg.sender] += amount;
    IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
  }

  function withdraw(uint256 amount) external {
    if (amount == 0) revert ZeroAmount();

    uint256 currentBalance = balance[msg.sender];
    if (currentBalance < amount) revert InsufficientAmount(currentBalance, amount);

    balance[msg.sender] = currentBalance - amount;
    IERC20(token).safeTransfer(msg.sender, amount);
  }
}
