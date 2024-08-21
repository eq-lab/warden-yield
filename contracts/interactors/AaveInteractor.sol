// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import {WadRayMath} from '@aave/core-v3/contracts/protocol/libraries/math/WadRayMath.sol';
import '@aave/core-v3/contracts/interfaces/IPool.sol';

import '../libraries/Errors.sol';
import '../interfaces/Aave/IAToken.sol';

/// @title abstract contract implementing the staking interaction with Aave Protocol
abstract contract AaveInteractor is Initializable {
  using SafeERC20 for IERC20;
  using WadRayMath for uint256;

  /// @custom:storage-location erc7201:eq-lab.storage.AaveInteractor
  struct AaveInteractorData {
    /// @dev address of Aave pool
    address aavePool;
    /// @dev not used since v2
    bool areWithdrawalsEnabled;
    /// @dev not used since v2
    mapping(address /* token */ => bool /* isAllowed */) allowedTokens;
    /// @dev token address used in stake/unstake operations
    address underlyingToken;
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

  /// @dev initialize method
  /// @param aavePool address of Aave pool which this contract will interact with
  /// @param tokens address of tokens which can be supplied to Aave pool via this contract
  function __AaveInteractor_init(address aavePool, address[] calldata tokens) internal onlyInitializing {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    if (aavePool == address(0)) revert Errors.ZeroAddress();
    $.aavePool = aavePool;

    uint256 tokensCount = tokens.length;
    for (uint256 i; i < tokensCount; ++i) {
      address token = tokens[i];
      uint256 coeff = IPool(aavePool).getReserveNormalizedIncome(token);
      if (coeff == 0) revert Errors.UnknownToken(token);

      $.allowedTokens[token] = true;
    }
  }

  function __AaveInteractor_initV2(address underlyingToken) internal onlyInitializing {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();

    if (!$.allowedTokens[underlyingToken]) revert Errors.NotAllowedToken(underlyingToken);
    $.underlyingToken = underlyingToken;
  }

  /// @dev method implementing 'stake' interaction with Aave pool
  /// @param amount amount of the supplied token
  /// @return scaledDepositAmount amount of Aave aToken received in staking process
  function _aaveStake(uint256 amount) internal returns (uint256 scaledDepositAmount) {
    if (amount == 0) revert Errors.ZeroAmount();

    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    address aavePool = $.aavePool;
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

    AaveInteractorData storage $ = _getAaveInteractorDataStorage();

    withdrawn = IPool($.aavePool).withdraw($.underlyingToken, amount, address(this));
    if (withdrawn < amount) revert Errors.InvalidAmount(amount, withdrawn);
  }

  /// @dev returns current balance of token supplied to Aave pool by this contract
  /// @param scaledAmount amount of the withdrawn token
  function _getBalanceFromScaled(uint256 scaledAmount) internal view returns (uint256) {
    return scaledAmount.rayMul(IPool(getAavePool()).getReserveNormalizedIncome(getUnderlyingToken()));
  }

  function _getScaledFromBalance(uint256 balanceAmount) internal view returns (uint256) {
    return balanceAmount.rayDiv(IPool(getAavePool()).getReserveNormalizedIncome(getUnderlyingToken()));
  }

  /// @notice returns address of Aave pool this contract interacts with
  function getAavePool() public view returns (address) {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    return $.aavePool;
  }

  /// @notice returns address of the underlying token
  function getUnderlyingToken() public view returns (address) {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    return $.underlyingToken;
  }
}
