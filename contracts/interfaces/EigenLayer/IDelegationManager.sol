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
  /**
   * @notice Caller delegates their stake to an operator.
   * @param operator The account (`msg.sender`) is delegating its assets to for use in serving applications built on EigenLayer.
   * @param approverSignatureAndExpiry Verifies the operator approves of this delegation
   * @param approverSalt A unique single use value tied to an individual signature.
   * @dev The approverSignatureAndExpiry is used in the event that:
   *          1) the operator's `delegationApprover` address is set to a non-zero value.
   *                  AND
   *          2) neither the operator nor their `delegationApprover` is the `msg.sender`, since in the event that the operator
   *             or their delegationApprover is the `msg.sender`, then approval is assumed.
   * @dev In the event that `approverSignatureAndExpiry` is not checked, its content is ignored entirely; it's recommended to use an empty input
   * in this case to save on complexity + gas costs
   */
  function delegateTo(
    address operator,
    SignatureWithExpiry memory approverSignatureAndExpiry,
    bytes32 approverSalt
  ) external;
}
