// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';

import '../libraries/Errors.sol';
import '../interfaces/EigenLayer/IDelegationManager.sol';
import '../interfaces/EigenLayer/IStrategyManager.sol';
import '../interfaces/EigenLayer/IStrategy.sol';
import '../interfaces/Lido/IStETH.sol';

/// @title abstract contract implementing the resstaking interaction with EigenLayer Protocol
abstract contract EigenLayerInteractor is Initializable {
  using SafeERC20 for IERC20;

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

  /// @dev 'EigenLayerInteractorData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.EigenLayerInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant EigenLayerInteractorDataStorageLocation =
    0xe36167a3404639da86a367e855838355c64e0a9aa7602a57452c5bbf07ac8c00;

  /// @dev returns storage slot of 'EigenLayerInteractorData' struct
  function _getEigenLayerInteractorDataStorage() internal pure returns (EigenLayerInteractorData storage $) {
    assembly {
      $.slot := EigenLayerInteractorDataStorageLocation
    }
  }

  /// @dev initialize method
  /// @param underlyingToken address of token deposited to EigenLayer strategy
  /// @param strategy address of EigenLayer strategy. Its underlying token is compared with the passed one
  /// @param strategyManager address of EigenLayer stategy manager. Checks if the passed strategy is whitelisted
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
    IERC20(data.underlyingToken).approve(data.strategyManager, underlyingTokenAmount);
    shares = IStrategyManager(data.strategyManager).depositIntoStrategy(
      IStrategy(data.strategy),
      IERC20(data.underlyingToken),
      underlyingTokenAmount
    );
  }
}
