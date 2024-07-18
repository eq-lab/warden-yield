// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';

import '../libraries/Errors.sol';
import '../interfaces/EigenLayer/IDelegationManager.sol';
import '../interfaces/EigenLayer/IStrategyManager.sol';
import '../interfaces/EigenLayer/IStrategy.sol';
import '../interfaces/Lido/IStETH.sol';

/// @title abstract contract implementing the restaking interaction with EigenLayer Protocol
abstract contract EigenLayerInteractor is Initializable {
  using SafeERC20 for IERC20;

  event EigenLayerWithdrawStart(uint64 indexed unstakeId, uint256 sharesToWithdraw);
  event EigenLayerWithdrawComplete(uint64 indexed unstakeId, uint256 withdrawnUnderlyingTokenAmount);

  /// @custom:storage-location erc7201:eq-lab.storage.EigenLayerInteractor
  struct EigenLayerInteractorData {
    /// @dev token deposited to EigenLayer strategy
    address underlyingToken;
    /// @dev address of EigenLayer strategy
    address strategy;
    /// @dev address of EigenLayer strategy manager
    address strategyManager;
    /// @dev address of EigenLayer delegation manager
    address delegationManager;
    /// @dev address of an operator who the restaked token will be delegated to
    address operator;
  }

  /// @custom:storage-location erc7201:eq-lab.storage.EigenLayerWithdrawQueue
  struct EigenLayerWithdrawQueue {
    /// @dev queue inner starting index. Increased after popping the first element
    uint128 start;
    /// @dev queue inner ending index. Increased after pushing the new element
    uint128 end;
    /// @dev queue items
    mapping(uint128 index => EigenLayerWithdrawQueueElement) items;
  }

  /// @dev struct used in view method to get withdrawal queue element by index
  struct EigenLayerWithdrawQueueElement {
    uint256 shares;
    uint32 blockNumber;
    uint64 unstakeId;
  }

  /// @dev 'EigenLayerInteractorData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.EigenLayerInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant EigenLayerInteractorDataStorageLocation =
    0xe36167a3404639da86a367e855838355c64e0a9aa7602a57452c5bbf07ac8c00;

  /// @dev 'EigenLayerWithdrawQueue' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.EigenLayerWithdrawQueue")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant EigenLayerWithdrawQueueStorageLocation =
    0x1ba446ce52667ee7efce266eb169f926c69ecd6f0d83ba53ae7bafe34f77a000;

  /// @dev returns storage slot of 'EigenLayerInteractorData' struct
  function _getEigenLayerInteractorDataStorage() internal pure returns (EigenLayerInteractorData storage $) {
    assembly {
      $.slot := EigenLayerInteractorDataStorageLocation
    }
  }

  /// @dev returns storage slot of 'EigenLayerInteractorData' struct
  function _getEigenLayerWithdrawQueueStorage() internal pure returns (EigenLayerWithdrawQueue storage $) {
    assembly {
      $.slot := EigenLayerWithdrawQueueStorageLocation
    }
  }

  /// @dev initialize method
  /// @param underlyingToken address of token deposited to EigenLayer strategy
  /// @param strategy address of EigenLayer strategy. Its underlying token is compared with the passed one
  /// @param strategyManager address of EigenLayer strategy manager. Checks if the passed strategy is whitelisted
  /// @param delegationManager address of EigenLayer delegation manager
  /// @param operator operator address who the restaked token will be delegated to. Gets verified in `delegationManager`
  function __EigenLayerInteractor_init(
    address underlyingToken,
    address strategy,
    address strategyManager,
    address delegationManager,
    address operator
  ) internal onlyInitializing {
    if (!IStrategyManager(strategyManager).strategyIsWhitelistedForDeposit(IStrategy(strategy)))
      revert Errors.WrongStrategy(strategy);
    if (address(IStrategy(strategy).underlyingToken()) != underlyingToken) revert Errors.UnknownToken(underlyingToken);
    if (!IDelegationManager(delegationManager).isOperator(operator)) revert Errors.WrongOperator(operator);

    SignatureWithExpiry memory defaultSignature;
    IDelegationManager(delegationManager).delegateTo(operator, defaultSignature, bytes32(0));

    EigenLayerInteractorData storage $ = _getEigenLayerInteractorDataStorage();
    $.underlyingToken = underlyingToken;
    $.strategy = strategy;
    $.strategyManager = strategyManager;
    $.delegationManager = delegationManager;
    $.operator = operator;
  }

  /// @dev EigenLayer restaking method
  /// @param underlyingTokenAmount amount of underlying tokens to be restaked
  /// @return shares amount of EigenLayer shares received in restaking process
  function _eigenLayerRestake(uint256 underlyingTokenAmount) internal returns (uint256 shares) {
    if (underlyingTokenAmount == 0) revert Errors.ZeroAmount();
    EigenLayerInteractorData memory data = _getEigenLayerInteractorDataStorage();
    IERC20(data.underlyingToken).forceApprove(data.strategyManager, underlyingTokenAmount);
    shares = IStrategyManager(data.strategyManager).depositIntoStrategy(
      IStrategy(data.strategy),
      IERC20(data.underlyingToken),
      underlyingTokenAmount
    );
  }

  /// @dev Withdraws underlying token from EigenLayer protocol
  /// @param sharesToWithdraw amount to withdraw represented in EigenLayer shares
  /// @param unstakeId unique id of the unstake
  function _eigenLayerWithdraw(uint64 unstakeId, uint256 sharesToWithdraw) internal {
    if (sharesToWithdraw <= _getEigenLayerMinSharesToWithdraw()) revert Errors.LowWithdrawalAmount(sharesToWithdraw);

    EigenLayerInteractorData memory data = _getEigenLayerInteractorDataStorage();

    IStrategy[] memory strategies = new IStrategy[](1);
    strategies[0] = IStrategy(data.strategy);

    uint256[] memory shares = new uint256[](1);
    shares[0] = sharesToWithdraw;

    IDelegationManager.QueuedWithdrawalParams[] memory params = new IDelegationManager.QueuedWithdrawalParams[](1);
    params[0] = IDelegationManager.QueuedWithdrawalParams({
      strategies: strategies,
      shares: shares,
      withdrawer: address(this)
    });

    IDelegationManager(data.delegationManager).queueWithdrawals(params);
    _enqueue(_getEigenLayerWithdrawQueueStorage(), unstakeId, sharesToWithdraw);
    emit EigenLayerWithdrawStart(unstakeId, sharesToWithdraw);
  }

  /// @dev completes if possible the oldest non-fulfilled withdrawal request
  /// @return unstakeId unique id of the unstake
  /// @return withdrawnAmount amount of underlyingToken received. Returns 0 if no request was completed
  function _eigenLayerReinit() internal returns (uint64 unstakeId, uint256 withdrawnAmount) {
    EigenLayerWithdrawQueue storage withdrawQueue = _getEigenLayerWithdrawQueueStorage();
    uint128 queueStart = withdrawQueue.start;
    uint128 queueEnd = withdrawQueue.end;
    if (queueStart == queueEnd) return (0, 0);

    EigenLayerInteractorData memory data = _getEigenLayerInteractorDataStorage();

    IStrategy[] memory strategies = new IStrategy[](1);
    IStrategy strategy = IStrategy(data.strategy);
    strategies[0] = strategy;

    EigenLayerWithdrawQueueElement memory queueItem = withdrawQueue.items[queueStart];
    uint256[] memory shares = new uint256[](1);
    shares[0] = queueItem.shares;

    IDelegationManager.Withdrawal memory withdrawal = IDelegationManager.Withdrawal({
      staker: address(this),
      delegatedTo: data.operator,
      withdrawer: address(this),
      nonce: queueStart,
      startBlock: queueItem.blockNumber,
      strategies: strategies,
      shares: shares
    });

    IERC20[] memory tokens = new IERC20[](1);
    tokens[0] = IERC20(data.underlyingToken);

    uint256 underlyingAmount = strategy.sharesToUnderlyingView(queueItem.shares);

    try IDelegationManager(data.delegationManager).completeQueuedWithdrawal(withdrawal, tokens, 0, true) {
      // TODO check this out -- seems like the eigenLayer + lido calculations combination can lose precision in both directions
      withdrawnAmount = underlyingAmount;
      unstakeId = queueItem.unstakeId;

      emit EigenLayerWithdrawComplete(queueItem.unstakeId, withdrawnAmount);
      _dequeue(withdrawQueue);
    } catch {}
  }

  /// @dev adds new withdraw request to the end of the EigenLayerWithdrawQueue
  function _enqueue(EigenLayerWithdrawQueue storage queue, uint64 unstakeId, uint256 sharesToWithdraw) private {
    uint128 queueEnd = queue.end;
    queue.items[queueEnd] = EigenLayerWithdrawQueueElement({
      unstakeId: unstakeId,
      shares: sharesToWithdraw,
      blockNumber: uint32(block.number)
    });
    unchecked {
      ++queue.end;
    }
  }

  /// @dev removes the first item from the EigenLayerWithdrawQueue after it's fulfilled
  function _dequeue(EigenLayerWithdrawQueue storage queue) private {
    uint128 queueStart = queue.start;
    delete queue.items[queueStart];
    unchecked {
      ++queue.start;
    }
  }

  /// @dev returns EigenLayerWithdrawQueueElement element by index
  function _getEigenLayerWithdrawalQueueElement(
    uint128 index
  ) internal view returns (EigenLayerWithdrawQueueElement memory) {
    EigenLayerWithdrawQueue storage queue = _getEigenLayerWithdrawQueueStorage();
    uint128 memoryIndex = queue.start + index;
    if (memoryIndex >= queue.end) revert Errors.NoElementWithIndex(index);

    return queue.items[memoryIndex];
  }

  /// @dev returns min amount allowed to be withdrawn from EigenLayer
  /// @dev returns 0, but can be overridden for usage in more complex cases
  function _getEigenLayerMinSharesToWithdraw() internal view virtual returns (uint256) {
    return 0;
  }
}
