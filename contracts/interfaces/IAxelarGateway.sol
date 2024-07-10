// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

///@title Interface for AxelarGateway
interface IAxelarGateway {
  function callContract(
    string calldata destinationChain,
    string calldata contractAddress,
    bytes calldata payload
  ) external;

  function callContractWithToken(
    string calldata destinationChain,
    string calldata contractAddress,
    bytes calldata payload,
    string calldata symbol,
    uint256 amount
  ) external;

  function validateContractCall(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes32 payloadHash
  ) external returns (bool);

  function validateContractCallAndMint(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes32 payloadHash,
    string calldata symbol,
    uint256 amount
  ) external returns (bool);

  function tokenAddresses(string memory symbol) external view returns (address);
}
