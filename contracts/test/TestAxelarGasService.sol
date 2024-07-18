// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../interfaces/Axelar/IAxelarGasService.sol';

contract TestAxelarGasService is IAxelarGasService {
  function payNativeGasForContractCallWithToken(
    address sender,
    string calldata destinationChain,
    string calldata destinationAddress,
    bytes calldata payload,
    string calldata symbol,
    uint256 amount,
    address refundAddress
  ) external payable override {}
}
