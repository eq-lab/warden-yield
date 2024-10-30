// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import {WadRayMath} from '@aave/core-v3/contracts/protocol/libraries/math/WadRayMath.sol';
import '@aave/core-v3/contracts/interfaces/IPool.sol';
import '@aave/core-v3/contracts/interfaces/IPoolAddressesProvider.sol';

import '../libraries/Errors.sol';
import '../interfaces/Aave/IAToken.sol';

/// @title abstract contract implementing the staking interaction with Aave Protocol
abstract contract AaveInteractor is Initializable {
  using SafeERC20 for IERC20;
  using WadRayMath for uint256;

  /// @custom:storage-location erc7201:eq-lab.storage.AaveInteractor
  struct AaveInteractorData {
    /// @dev replaced by aavePoolProvider in v2
    address aavePool;
    /// @dev not used since v2
    bool areWithdrawalsEnabled;
    /// @dev not used since v2
    mapping(address /* token */ => bool /* isAllowed */) allowedTokens;
    /// @dev token address used in stake/unstake operations
    address underlyingToken;
    /// @dev the recommended way to get aave pool address
    address aavePoolProvider;
  }

  /// @dev 'AaveInteractorData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.AaveInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant AaveInteractorDataStorageLocation =
    0x44276a98a797c93c865e1a1b83c2084ae09aae5cb934985d7d8c98e53b664200;

  /// @dev returns storage slot of 'AaveInteractorData' struct
  function _getAaveInteractorDataStorage() private pure returns (AaveInteractorData storage $) {
    assembly {
      $.slot := AaveInteractorDataStorageLocation
    }
  }

  /// @notice initialize method used during deployment from scratch
  /// @param aavePoolProvider address of Aave pool provider which this contract will interact with
  /// @param underlyingToken address of token which can be supplied to Aave pool via this contract
  function __AaveInteractor_init(address aavePoolProvider, address underlyingToken) internal onlyInitializing {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    if (aavePoolProvider == address(0)) revert Errors.ZeroAddress();

    address aavePool = IPoolAddressesProvider(aavePoolProvider).getPool();
    if (IPool(aavePool).getReserveNormalizedIncome(underlyingToken) == 0) revert Errors.UnknownToken(underlyingToken);

    $.aavePoolProvider = aavePoolProvider;
    $.underlyingToken = underlyingToken;
  }

  /// @notice initialize method used during upgrade
  /// @param aavePoolProvider address of Aave pool provider which this contract will interact with
  /// @param underlyingToken address of token which can be supplied to Aave pool via this contract
  /// @dev both pool provider and token must be corresponding to the previous setup
  function __AaveInteractor_initV2(address aavePoolProvider, address underlyingToken) internal onlyInitializing {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();

    if (!$.allowedTokens[underlyingToken]) revert Errors.NotAllowedToken(underlyingToken);

    address aavePool = IPoolAddressesProvider(aavePoolProvider).getPool();
    if (aavePool != $.aavePool) revert Errors.WrongPoolProvider(aavePoolProvider, aavePool);

    $.underlyingToken = underlyingToken;
    $.aavePoolProvider = aavePoolProvider;
    delete $.aavePool;
    delete $.allowedTokens[underlyingToken];
    delete $.areWithdrawalsEnabled;
  }

  /// @dev method implementing 'stake' interaction with Aave pool
  /// @param amount amount of the supplied token
  /// @return scaledDepositAmount amount of Aave aToken received in staking process
  function _aaveStake(uint256 amount) internal returns (uint256 scaledDepositAmount) {
    if (amount == 0) revert Errors.ZeroAmount();

    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    address aavePool = _getAavePool();
    address token = $.underlyingToken;
    address aToken = IPool(aavePool).getReserveData(token).aTokenAddress;

    uint256 totalBalanceScaledBefore = IAToken(aToken).scaledBalanceOf(address(this));
    IERC20(token).forceApprove(aavePool, amount);
    IPool(aavePool).supply(token, amount, address(this), 0);

    scaledDepositAmount = IAToken(aToken).scaledBalanceOf(address(this)) - totalBalanceScaledBefore;
    if (scaledDepositAmount == 0) revert Errors.ZeroAmount();
  }

  /// @dev method implementing 'withdraw' interaction with Aave pool
  /// @param amount amount of the withdrawn token
  function _aaveWithdraw(uint256 amount) internal returns (uint256 withdrawn) {
    if (amount == 0) revert Errors.ZeroAmount();

    withdrawn = IPool(_getAavePool()).withdraw(_getUnderlyingToken(), amount, address(this));
    if (withdrawn < amount) revert Errors.InvalidAmount(amount, withdrawn);
  }

  function _getAavePoolProvider() internal view returns (address) {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    return $.aavePoolProvider;
  }

  function _getAavePool() internal view returns (address) {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    return IPoolAddressesProvider($.aavePoolProvider).getPool();
  }

  function _getUnderlyingToken() internal view returns (address) {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    return $.underlyingToken;
  }

  /// @dev returns current balance of token supplied to Aave pool by this contract
  /// @param scaledAmount amount of the withdrawn token
  function _getBalanceFromScaled(uint256 scaledAmount) internal view returns (uint256) {
    return scaledAmount.rayMul(IPool(_getAavePool()).getReserveNormalizedIncome(_getUnderlyingToken()));
  }

  function _getScaledFromBalance(uint256 balanceAmount) internal view returns (uint256) {
    return balanceAmount.rayDiv(IPool(_getAavePool()).getReserveNormalizedIncome(_getUnderlyingToken()));
  }

  /// @notice returns address of Aave pool this contract interacts with
  function getAavePoolProvider() external view returns (address) {
    return _getAavePoolProvider();
  }

  /// @notice returns address of the underlying token
  function getUnderlyingToken() external view returns (address) {
    return _getUnderlyingToken();
  }
}
