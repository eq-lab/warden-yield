// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.26;

library Errors {
    error ZeroAddress();
    error NotAllowedToken(address);
    error TokenAlreadyAllowed(address);
    error UnknownToken(address);
    error ZeroAmount();
    error WithdrawalsDisabled();
    error InvalidAmount(uint256 expected, uint256 actual);
}
