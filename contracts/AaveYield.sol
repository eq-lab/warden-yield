// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './libraries/Errors.sol';
import './interactors/AaveInteractor.sol';
import './YieldStorage.sol';

contract AaveYield is UUPSUpgradeable, Ownable2StepUpgradeable, AaveInteractor, YieldStorage {
  event Stake(address indexed user, address indexed token, uint256 stakedAmount, uint256 shares);
  event Withdraw(address indexed user, address indexed token, uint256 withdrawAmount);
  event EnableWithdrawals();
  event DisableWithdrawals();

  function initialize(address aavePool, address[] calldata tokens) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __AaveInteractor_init(aavePool, tokens);
  }

  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function stake(address token, uint256 amount) external payable returns (uint256 shares) {
    shares = _stake(token, amount);
    _addStake(msg.sender, token, amount, shares);

    emit Stake(msg.sender, token, amount, shares);
  }

  function withdraw(address token) external returns (uint256 withdrawAmount) {
    if (!isWithdrawalsEnabled()) revert Errors.WithdrawalsDisabled();

    withdrawAmount = getAvailableToWithdraw(msg.sender, token);

    _withdraw(token, withdrawAmount);
    _removeStake(msg.sender, token);

    emit Withdraw(msg.sender, token, withdrawAmount);
  }

  function enableWithdrawals() external onlyOwner {
    _enableWithdrawals();
    emit EnableWithdrawals();
  }

  function disableWithdrawals() external onlyOwner {
    _disableWithdrawals();
    emit DisableWithdrawals();
  }

  function allowTokens(address[] calldata tokens) external onlyOwner {
    uint256 tokensCount = tokens.length;
    for (uint256 i = 0; i < tokensCount; i++) {
      address token = tokens[i];
      if (getTokenAllowance(token)) revert Errors.TokenAlreadyAllowed(token);

      address aavePool = getAavePool();
      uint256 coeff = IAavePool(aavePool).getReserveNormalizedIncome(token);
      if (coeff == 0) revert Errors.UnknownToken(token);

      _setTokenAllowance(token, true);
    }
  }

  function disallowTokens(address[] calldata tokens) external onlyOwner {
    uint256 tokensCount = tokens.length;
    for (uint256 i = 0; i < tokensCount; i++) {
      address token = tokens[i];
      if (getTokenAllowance(token)) revert Errors.TokenAlreadyAllowed(token);

      address aavePool = getAavePool();
      uint256 coeff = IAavePool(aavePool).getReserveNormalizedIncome(token);
      if (coeff == 0) revert Errors.UnknownToken(token);

      _setTokenAllowance(token, true);
    }
  }

  function getAvailableToWithdraw(address user, address token) public view returns (uint256 availableToWithdraw) {
    uint256 scaledDeposit = userShares(user, token);
    availableToWithdraw = _getBalanceFromScaled(scaledDeposit, token);
  }
}
