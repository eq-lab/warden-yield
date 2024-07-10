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
  struct QueuedWithdrawalParams {
    // Array of strategies that the QueuedWithdrawal contains
    IStrategy[] strategies;
    // Array containing the amount of shares in each Strategy in the `strategies` array
    uint256[] shares;
    // The address of the withdrawer
    address withdrawer;
  }

  struct Withdrawal {
    // The address that originated the Withdrawal
    address staker;
    // The address that the staker was delegated to at the time that the Withdrawal was created
    address delegatedTo;
    // The address that can complete the Withdrawal + will receive funds when completing the withdrawal
    address withdrawer;
    // Nonce used to guarantee that otherwise identical withdrawals have unique hashes
    uint256 nonce;
    // Block number when the Withdrawal was created
    uint32 startBlock;
    // Array of strategies that the Withdrawal contains
    IStrategy[] strategies;
    // Array containing the amount of shares in each Strategy in the `strategies` array
    uint256[] shares;
  }

  event OperatorSharesIncreased(address indexed operator, address staker, IStrategy strategy, uint256 shares);

  function delegateTo(
    address operator,
    SignatureWithExpiry memory approverSignatureAndExpiry,
    bytes32 approverSalt
  ) external;

  function queueWithdrawals(
    QueuedWithdrawalParams[] calldata queuedWithdrawalParams
  ) external returns (bytes32[] memory withdrawalRoots);

  function completeQueuedWithdrawal(
    Withdrawal calldata withdrawal,
    IERC20[] calldata tokens,
    uint256 middlewareTimesIndex,
    bool receiveAsTokens
  ) external;

  function completeQueuedWithdrawals(
    Withdrawal[] calldata withdrawals,
    IERC20[][] calldata tokens,
    uint256[] calldata middlewareTimesIndexes,
    bool[] calldata receiveAsTokens
  ) external;

  function isOperator(address operator) external view returns (bool);

  function delegatedTo(address staker) external view returns (address);

  function MAX_WITHDRAWAL_DELAY_BLOCKS() external view returns (uint256);
}
