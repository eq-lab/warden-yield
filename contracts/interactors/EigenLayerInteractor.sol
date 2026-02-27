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
  event EigenLayerWithdrawComplete(uint256 withdrawnUnderlyingTokenAmount);

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

  /// @custom:storage-location erc7201:eq-lab.storage.EigenLayerWithdraw
  struct EigenLayerWithdraw {
    uint256 shares;
    uint32 blockNumber;
  }

  /// @dev 'EigenLayerInteractorData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.EigenLayerInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant EigenLayerInteractorDataStorageLocation =
    0xe36167a3404639da86a367e855838355c64e0a9aa7602a57452c5bbf07ac8c00;

  /// @dev 'EigenLayerWithdrawQueue' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.EigenLayerWithdraw")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant EigenLayerWithdrawStorageLocation =
    0xd0caed1a8e9a35ef382659d2ac8f58e75b083adac6d1993bb70ae5fc77aeb200;

  /// @dev returns storage slot of 'EigenLayerInteractorData' struct
  function _getEigenLayerInteractorDataStorage() internal pure returns (EigenLayerInteractorData storage $) {
    assembly {
      $.slot := EigenLayerInteractorDataStorageLocation
    }
  }

  /// @dev returns storage slot of 'EigenLayerInteractorData' struct
  function _getEigenLayerWithdrawStorage() internal pure returns (EigenLayerWithdraw storage $) {
    assembly {
      $.slot := EigenLayerWithdrawStorageLocation
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

  /// @dev Withdraws underlying token from EigenLayer protocol
  function _eigenLayerFullWithdraw() internal {
    EigenLayerInteractorData storage data = _getEigenLayerInteractorDataStorage();

    IStrategy strategy = IStrategy(data.strategy);
    uint256 sharesToWithdraw = strategy.shares(address(this));

    IStrategy[] memory strategies = new IStrategy[](1);
    strategies[0] = strategy;

    uint256[] memory shares = new uint256[](1);
    shares[0] = sharesToWithdraw;

    IDelegationManager.QueuedWithdrawalParams[] memory params = new IDelegationManager.QueuedWithdrawalParams[](1);
    params[0] = IDelegationManager.QueuedWithdrawalParams({
      strategies: strategies,
      shares: shares,
      withdrawer: address(this)
    });

    IDelegationManager(data.delegationManager).queueWithdrawals(params);

    EigenLayerWithdraw storage withdraw = _getEigenLayerWithdrawStorage();
    withdraw.blockNumber = uint32(block.number);
    withdraw.shares = sharesToWithdraw;

    emit EigenLayerWithdrawStart(sharesToWithdraw);
  }

  /// @dev completes if possible the oldest non-fulfilled withdrawal request
  /// @return withdrawnAmount amount of underlyingToken received. Returns 0 if no request was completed
  function _eigenLayerCompleteWithdrawal() internal returns (uint256 withdrawnAmount) {
    EigenLayerWithdraw storage withdraw = _getEigenLayerWithdrawStorage();

    if (withdraw.shares == 0) revert Errors.NoActiveWithdrawal();

    EigenLayerInteractorData storage data = _getEigenLayerInteractorDataStorage();

    IStrategy[] memory strategies = new IStrategy[](1);
    IStrategy strategy = IStrategy(data.strategy);
    strategies[0] = strategy;

    uint256[] memory shares = new uint256[](1);
    shares[0] = withdraw.shares;

    IDelegationManager.Withdrawal memory withdrawal = IDelegationManager.Withdrawal({
      staker: address(this),
      delegatedTo: data.operator,
      withdrawer: address(this),
      nonce: 0,
      startBlock: withdraw.blockNumber,
      strategies: strategies,
      shares: shares
    });

    IERC20 underlyingToken = IERC20(data.underlyingToken);

    IERC20[] memory tokens = new IERC20[](1);
    tokens[0] = underlyingToken;

    uint256 balanceBefore = underlyingToken.balanceOf(address(this));

    IDelegationManager(data.delegationManager).completeQueuedWithdrawal(withdrawal, tokens, true);

    withdrawnAmount = underlyingToken.balanceOf(address(this)) - balanceBefore;
    emit EigenLayerWithdrawComplete(withdrawnAmount);

    withdraw.blockNumber = 0;
    withdraw.shares = 0;
  }

  function getEigenLayerWithdrawalData() public view returns (uint32, uint256) {
    EigenLayerWithdraw storage $ = _getEigenLayerWithdrawStorage();
    return ($.blockNumber, $.shares);
  }
}
