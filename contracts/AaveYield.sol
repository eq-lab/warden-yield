// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './libraries/Errors.sol';
import './interactors/AaveInteractor.sol';
import './YieldStorage.sol';
import './interfaces/IAaveYield.sol';

contract AaveYield is UUPSUpgradeable, Ownable2StepUpgradeable, AaveInteractor, YieldStorage, IAaveYield {
  /// @notice initialize function used during contract deployment
  /// @param aavePool address of a Aave pool
  /// @param tokens array with addresses of tokens which will be used in the Aave pool
  function initialize(address aavePool, address[] calldata tokens) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __AaveInteractor_init(aavePool, tokens);
  }

  /// @dev method called during the contract upgrade
  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}


  /// @inheritdoc IAaveYield
  function stake(address token, uint256 amount, string calldata userWardenAddress) external returns (uint256 shares) {
    shares = _aaveStake(token, amount);
    _addStake(msg.sender, token, amount, shares);
    _addWardenAddress(msg.sender, userWardenAddress);

    emit Stake(msg.sender, token, amount, shares);
  }

  /// @inheritdoc IAaveYield
  function withdraw(address token) external returns (uint256 withdrawAmount) {
    if (!areWithdrawalsEnabled()) revert Errors.WithdrawalsDisabled();

    withdrawAmount = getAvailableToWithdraw(msg.sender, token);

    _aaveWithdraw(token, withdrawAmount);
    _removeStake(msg.sender, token);

    emit Withdraw(msg.sender, token, withdrawAmount);
  }

  /// @notice enables withdraws
  /// @dev in the first version withdraws are disabled by default
  function enableWithdrawals() external onlyOwner {
    _enableWithdrawals();
    emit EnableWithdrawals();
  }

  /// @notice disables withdraws
  /// @dev in the first version withdraws are disabled by default
  function disableWithdrawals() external onlyOwner {
    _disableWithdrawals();
    emit DisableWithdrawals();
  }

  /// @notice allows passed tokens usage in 'stake' call
  /// @dev checks if tokens are included in 'AavePool.getReservesList()'
  function allowTokens(address[] calldata tokens) external onlyOwner {
    uint256 tokensCount = tokens.length;
    for (uint256 i; i < tokensCount; ++i) {
      address token = tokens[i];
      if (getTokenAllowance(token)) revert Errors.TokenAlreadyAllowed(token);

      address aavePool = getAavePool();
      uint256 coeff = IPool(aavePool).getReserveNormalizedIncome(token);
      if (coeff == 0) revert Errors.UnknownToken(token);

      _setTokenAllowance(token, true);
    }
  }

  /// @notice disallows passed tokens usage in 'stake' call
  function disallowTokens(address[] calldata tokens) external onlyOwner {
    uint256 tokensCount = tokens.length;
    for (uint256 i; i < tokensCount; ++i) {
      address token = tokens[i];
      if (!getTokenAllowance(token)) revert Errors.TokenAlreadyDisallowed(token);

      _setTokenAllowance(token, false);
    }
  }

  /// @inheritdoc IAaveYield
  function getAvailableToWithdraw(address user, address token) public view returns (uint256 availableToWithdraw) {
    uint256 scaledDeposit = userShares(user, token);
    availableToWithdraw = _getBalanceFromScaled(scaledDeposit, token);
  }
}
