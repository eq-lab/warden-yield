// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

/**
 * @title IAxelarGasService Interface
 * @notice This is an interface for the AxelarGasService contract which manages gas payments
 * and refunds for cross-chain communication on the Axelar network.
 * @dev Copied from https://github.com/axelarnetwork/axelar-gmp-sdk-solidity/blob/main/contracts/interfaces/IAxelarGasService.sol
 */
interface IAxelarGasService {
  /**
   * @notice Pay for gas using native currency for a contract call with tokens on a destination chain.
   * @dev This function is called on the source chain before calling the gateway to execute a remote contract.
   * @param sender The address making the payment
   * @param destinationChain The target chain where the contract call with tokens will be made
   * @param destinationAddress The target address on the destination chain
   * @param payload Data payload for the contract call with tokens
   * @param symbol The symbol of the token to be sent with the call
   * @param amount The amount of tokens to be sent with the call
   * @param refundAddress The address where refunds, if any, should be sent
   */
  function payNativeGasForContractCallWithToken(
    address sender,
    string calldata destinationChain,
    string calldata destinationAddress,
    bytes calldata payload,
    string calldata symbol,
    uint256 amount,
    address refundAddress
  ) external payable;
}
