// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import './IStrategy.sol';
import './IStrategyManager.sol';

struct SignatureWithExpiry {
  // the signature itself, formatted as a single bytes object
  bytes signature;
  // the expiration timestamp (UTC) of the signature
  uint256 expiry;
}

/// @dev https://github.com/Layr-Labs/eigenlayer-contracts/blob/dev/src/contracts/interfaces/IDelegationManager.sol
interface IDelegationManager {
  event OperatorSharesIncreased(address indexed operator, address staker, IStrategy strategy, uint256 shares);

  function delegateTo(
    address operator,
    SignatureWithExpiry memory approverSignatureAndExpiry,
    bytes32 approverSalt
  ) external;

  function isOperator(address operator) external view returns (bool);

  function delegatedTo(address staker) external view returns (address);
}
