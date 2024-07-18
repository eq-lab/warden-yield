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

  function callContract(
    string calldata destinationChain,
    string calldata contractAddress,
    bytes calldata payload
  ) external override {
    callContractPayload = payload;
  }

  function callContractWithToken(
    string calldata destinationChain,
    string calldata contractAddress,
    bytes calldata payload,
    string calldata symbol,
    uint256 amount
  ) external override {
    callContractWithTokenPayload = payload;
  }

  function validateContractCall(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes32 payloadHash
  ) external override returns (bool) {
    return _isValidCall;
  }

  function validateContractCallAndMint(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes32 payloadHash,
    string calldata symbol,
    uint256 amount
  ) external override returns (bool) {
    return _isValidCall;
  }

  function tokenAddresses(string memory symbol) external view override returns (address) {
    return _tokenAddresses[symbol];
  }
}
