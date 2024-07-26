// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../interfaces/Axelar/IAxelarGateway.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';

contract TestAxelarGateway is IAxelarGateway {
  using SafeERC20 for IERC20;

  event ContractCall(
    address indexed sender,
    string destinationChain,
    string destinationContractAddress,
    bytes32 indexed payloadHash,
    bytes payload
  );

  event ContractCallWithToken(
    address indexed sender,
    string destinationChain,
    string destinationContractAddress,
    bytes32 indexed payloadHash,
    bytes payload,
    string symbol,
    uint256 amount
  );

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
    string calldata destinationContractAddress,
    bytes calldata payload
  ) external override {
    callContractPayload = payload;

    emit ContractCall(msg.sender, destinationChain, destinationContractAddress, keccak256(payload), payload);
  }

  function callContractWithToken(
    string calldata destinationChain,
    string calldata destinationContractAddress,
    bytes calldata payload,
    string calldata symbol,
    uint256 amount
  ) external override {
    address token = _tokenAddresses[symbol];
    require(token != address(0), 'DEBUG');
    IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
    callContractWithTokenPayload = payload;

    emit ContractCallWithToken(
      msg.sender,
      destinationChain,
      destinationContractAddress,
      keccak256(payload),
      payload,
      symbol,
      amount
    );
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
