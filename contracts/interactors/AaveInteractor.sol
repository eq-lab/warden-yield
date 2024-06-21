// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import {WadRayMath} from '@aave/core-v3/contracts/protocol/libraries/math/WadRayMath.sol';
import '@aave/core-v3/contracts/interfaces/IPool.sol';

import '../libraries/Errors.sol';
import '../interfaces/Aave/IAToken.sol';

abstract contract AaveInteractor is Initializable {
  using SafeERC20 for IERC20;
  using WadRayMath for uint256;

  /// @custom:storage-location erc7201:eq-lab.storage.AaveInteractor
  struct AaveInteractorData {
    address aavePool;
    bool areWithdrawalsEnabled;
    mapping(address /* token */ => bool /* isAllowed */) allowedTokens;
  }

  // keccak256(abi.encode(uint256(keccak256("eq-lab.storage.AaveInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant AaveInteractorDataStorageLocation =
    0x44276a98a797c93c865e1a1b83c2084ae09aae5cb934985d7d8c98e53b664200;

  function _getAaveInteractorDataStorage() private pure returns (AaveInteractorData storage $) {
    assembly {
      $.slot := AaveInteractorDataStorageLocation
    }
  }

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

  function _stake(address token, uint256 amount) internal returns (uint256 scaledDepositAmount) {
    if (token == address(0)) revert Errors.ZeroAddress();
    if (amount == 0) revert Errors.ZeroAmount();
    if (!getTokenAllowance(token)) revert Errors.NotAllowedToken(token);

    IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

    address aavePool = getAavePool();
    address aToken = IPool(aavePool).getReserveData(token).aTokenAddress;

    uint256 totalBalanceScaledBefore = IAToken(aToken).scaledBalanceOf(address(this));
    IERC20(token).approve(aavePool, amount);
    IPool(aavePool).supply(token, amount, address(this), 0);

    scaledDepositAmount = IAToken(aToken).scaledBalanceOf(address(this)) - totalBalanceScaledBefore;
    if (scaledDepositAmount == 0) revert Errors.ZeroAmount();
  }

  function _withdraw(address token, uint256 amount) internal {
    if (token == address(0)) revert Errors.ZeroAddress();
    if (!getTokenAllowance(token)) revert Errors.NotAllowedToken(token);

    address aavePool = getAavePool();
    address aToken = IPool(aavePool).getReserveData(token).aTokenAddress;
    if (aToken == address(0)) revert Errors.UnknownToken(token);

    uint256 totalBalanceScaledBefore = IAToken(aToken).scaledBalanceOf(address(this));
    uint256 withdrawAmount = IPool(aavePool).withdraw(token, amount, msg.sender);
    if (withdrawAmount != amount) revert Errors.InvalidAmount(amount, withdrawAmount);

    uint256 scaledWithdrawAmount = totalBalanceScaledBefore - IAToken(aToken).scaledBalanceOf(address(this));
    if (scaledWithdrawAmount == 0) revert Errors.ZeroAmount();
  }

  function _getBalanceFromScaled(uint256 scaledAmount, address token) internal view returns (uint256) {
    return scaledAmount.rayMul(IPool(getAavePool()).getReserveNormalizedIncome(token));
  }

  function getAavePool() public view returns (address) {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    return $.aavePool;
  }

  function _setAavePool(address aavePool) internal {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    $.aavePool = aavePool;
  }

  function areWithdrawalsEnabled() public view returns (bool) {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    return $.areWithdrawalsEnabled;
  }

  function _enableWithdrawals() internal {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    $.areWithdrawalsEnabled = true;
  }

  function _disableWithdrawals() internal {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    $.areWithdrawalsEnabled = false;
  }

  function getTokenAllowance(address token) public view returns (bool) {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    return $.allowedTokens[token];
  }

  function _setTokenAllowance(address token, bool enabled) internal {
    AaveInteractorData storage $ = _getAaveInteractorDataStorage();
    $.allowedTokens[token] = enabled;
  }
}
