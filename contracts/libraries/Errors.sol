// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.26;

library Errors {
  error ZeroAddress();
  error NotAllowedToken(address);
  error TokenAlreadyAllowed(address);
  error TokenAlreadyDisallowed(address);
  error UnknownToken(address);
  error ZeroAmount();
  error WithdrawalsDisabled();
  error InvalidAmount(uint256 expected, uint256 actual);
  error WrongMsgValue(uint256 msgValue, uint256 input);
  error WrongStrategy(address);
  error WrongOperator(address);
  error NotWETH9(address);
}
