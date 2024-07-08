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

  event EigenLayerWithdrawStart(uint256 sharesToWithdraw);
  event EigenLayerWithdrawComplete(uint256 withdrawnShares);

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
    uint256 start;
    uint256 end;
    mapping(uint256 index => uint256) shares;
    mapping(uint256 index => uint32) blockNumber;
  }

  /// @dev 'EigenLayerInteractorData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.EigenLayerInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant EigenLayerInteractorDataStorageLocation =
    0xe36167a3404639da86a367e855838355c64e0a9aa7602a57452c5bbf07ac8c00;

  /// @dev 'EigenLayerWithdrawQueue' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.EigenLayerWithdrawQueue")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant EigenLayerWithdrawQueueStorageLocation =
    0x1ba446ce52667ee7efce266eb169f926c69ecd6f0d83ba53ae7bafe34f77a000;

  modifier eigenLayerReinit() {
    _eigenLayerReinit();
    _;
  }

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
  function _eigenLayerRestake(uint256 underlyingTokenAmount) internal eigenLayerReinit returns (uint256 shares) {
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
  function _eigenLayerWithdraw(uint256 sharesToWithdraw) internal eigenLayerReinit {
    if (sharesToWithdraw == 0) revert Errors.ZeroAmount();

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
    _queuePush(_getEigenLayerWithdrawQueueStorage(), sharesToWithdraw);
    emit EigenLayerWithdrawStart(sharesToWithdraw);
  }

  /// @dev This method is called 
  function _eigenLayerReinit() internal {
    EigenLayerWithdrawQueue storage withdrawQueue = _getEigenLayerWithdrawQueueStorage();
    uint256 queueStart = withdrawQueue.start;
    uint256 queueEnd = withdrawQueue.end;
    if (queueStart == queueEnd) return;

    EigenLayerInteractorData memory data = _getEigenLayerInteractorDataStorage();

    IStrategy[] memory strategies = new IStrategy[](1);
    strategies[0] = IStrategy(data.strategy);

    uint256[] memory shares = new uint256[](1);
    shares[0] = withdrawQueue.shares[queueStart];

    IDelegationManager.Withdrawal memory withdrawal = IDelegationManager.Withdrawal({
      staker: address(this),
      delegatedTo: data.operator,
      withdrawer: address(this),
      nonce: queueStart,
      startBlock: withdrawQueue.blockNumber[queueStart],
      strategies: strategies,
      shares: shares
    });

    IERC20[] memory tokens = new IERC20[](1);
    tokens[0] = IERC20(data.underlyingToken);

    try IDelegationManager(data.delegationManager).completeQueuedWithdrawal(withdrawal, tokens, 0, true) {
      _queuePop(withdrawQueue);
      emit EigenLayerWithdrawComplete(shares[0]);
    } catch {}
  }

  /// @dev adds new withdraw request to the end of the EigenLayerWithdrawQueue
  function _queuePush(EigenLayerWithdrawQueue storage queue, uint256 sharesToWithdraw) private {
    uint256 queueEnd = queue.end;
    queue.blockNumber[queueEnd] = uint32(block.number);
    queue.shares[queueEnd] = sharesToWithdraw;
    unchecked {
      ++queue.end;
    }
  }

  /// @dev pops the first request from the EigenLayerWithdrawQueue after it's fulfilled
  function _queuePop(EigenLayerWithdrawQueue storage queue) private {
    uint256 queueStart = queue.start;
    delete queue.blockNumber[queueStart];
    delete queue.shares[queueStart];
    unchecked {
      ++queue.start;
    }
  }
}
