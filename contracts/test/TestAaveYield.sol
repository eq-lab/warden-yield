// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.26;

import '../AaveYield.sol';

contract TestAaveYield is AaveYield {
  /// @notice method to withdraw token from Aave pool
  /// @dev in the first version withdraws are disabled by default
  /// @param token address of a token to be withdrawn
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

  /// @notice disallows passed tokens usage in 'stake' call
  function disallowTokens(address[] calldata tokens) external onlyOwner {
    uint256 tokensCount = tokens.length;
    for (uint256 i; i < tokensCount; ++i) {
      address token = tokens[i];
      if (!getTokenAllowance(token)) revert Errors.TokenAlreadyDisallowed(token);

      _setTokenAllowance(token, false);
    }
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
}
