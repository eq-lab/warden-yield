// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../interfaces/Axelar/IAxelarGateway.sol';

contract TestAxelarGateway is IAxelarGateway {
  bool public _isValidCall = true;

  mapping(string symbol => address tokenAddress) _tokenAddresses;

  bytes public callContractWithTokenPayload;

  bytes public callContractPayload;

  function setIsValidCall(bool isValidCall) external {
    _isValidCall = isValidCall;
  }

  function resetPayload() external {
    callContractPayload = '';
    callContractWithTokenPayload = '';
  }

  function addTokenAddress(string memory symbol, address tokenAddress) external {
    _tokenAddresses[symbol] = tokenAddress;
  }

  function callContract(string calldata, string calldata, bytes calldata payload) external override {
    callContractPayload = payload;
  }

  function callContractWithToken(
    string calldata,
    string calldata,
    bytes calldata payload,
    string calldata,
    uint256
  ) external override {
    callContractWithTokenPayload = payload;
  }

  function validateContractCall(
    bytes32,
    string calldata,
    string calldata,
    bytes32
  ) external view override returns (bool) {
    return _isValidCall;
  }

  function validateContractCallAndMint(
    bytes32,
    string calldata,
    string calldata,
    bytes32,
    string calldata,
    uint256
  ) external view override returns (bool) {
    return _isValidCall;
  }

  function tokenAddresses(string memory symbol) external view override returns (address) {
    return _tokenAddresses[symbol];
  }
}
