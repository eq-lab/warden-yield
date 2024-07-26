// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './libraries/Errors.sol';
import './interactors/AaveInteractor.sol';
import './YieldStorage.sol';
import './interfaces/IAaveYield.sol';
import './WardenHandler.sol';

contract AaveYield is
  UUPSUpgradeable,
  Ownable2StepUpgradeable,
  AaveInteractor,
  YieldStorage,
  IAaveYield,
  WardenHandler
{
  // /// @notice initialize function used during contract deployment
  // /// @param aavePool address of a Aave pool
  // /// @param tokens array with addresses of tokens which will be used in the Aave pool
  // function initialize(address aavePool, address[] calldata tokens) external initializer {
  //   __Ownable_init(msg.sender);
  //   __UUPSUpgradeable_init();
  //   __AaveInteractor_init(aavePool, tokens);
  // }

  function initializeV2(
    address underlyingToken,
    address axelarGateway,
    address axelarGasService,
    string calldata wardenChain,
    string calldata wardenContractAddress
  ) external reinitializer(2) {
    __YieldStorage_initV2(underlyingToken);
    __AaveInteractor_initV2(underlyingToken);
    __WardenHandler_init(axelarGateway, axelarGasService, wardenChain, wardenContractAddress);

    // TODO: add lpAmount totalSupply initial value
  }

  /// @dev method called during the contract upgrade
  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  /// @dev Convert shares (Aave shares) to lp amount
  function _sharesToLpAmount(uint256 sharesAmount) internal pure returns (uint256) {
    //TODO: implement
    return sharesAmount;
  }

  /// @dev Convert lp amount to shares (Aave shares)
  function _lpAmountToShares(uint256 lpAmount) internal pure returns (uint256) {
    //TODO: implement
    return lpAmount;
  }

  /// @inheritdoc IAaveYield
  function stake(uint64 stakeId, uint256 amount) external returns (uint256 lpAmount) {
    require(msg.sender == address(this));
    uint256 shares = _aaveStake(amount);
    _addStake(shares);

    // TODO: add lpAmount calculation
    lpAmount = _sharesToLpAmount(shares);

    emit Stake(stakeId, amount, lpAmount);
  }

  function unstake(uint64 unstakeId, uint256 lpAmount) external returns (uint256 withdrawn) {
    require(msg.sender == address(this));

    uint256 sharesAmount = _lpAmountToShares(lpAmount);
    uint256 withdrawAmount = _getBalanceFromScaled(sharesAmount);
    withdrawn = _aaveWithdraw(withdrawAmount);
    _removeStake(sharesAmount);

    emit Unstake(unstakeId, withdrawn);
  }

  /// @notice converts amount of passed token to the shares
  function underlyingToShares(uint256 amount) external view returns (uint256) {
    return _getScaledFromBalance(amount);
  }

  /// @notice converts shares of passed token to its amount
  function sharesToUnderlying(uint256 shares) external view returns (uint256) {
    return _getBalanceFromScaled(shares);
  }

  /*** WardenHandler ***/

  function _handleStakeRequest(
    uint64 stakeId,
    uint256 amountToStake
  ) internal virtual override returns (StakeResult memory result) {
    result = WardenHandler.StakeResult({
      status: WardenHandler.Status.Failed,
      lpAmount: 0,
      unstakeTokenAmount: 0,
      reinitUnstakeId: 0
    });

    try this.stake(stakeId, amountToStake) returns (uint256 lpAmount) {
      result.status = WardenHandler.Status.Success;
      result.lpAmount = uint128(lpAmount);
    } catch (bytes memory reason) {
      emit RequestFailed(ActionType.Stake, stakeId, reason);
    }
  }

  function _handleUnstakeRequest(
    uint64 unstakeId,
    uint128 lpAmount
  ) internal virtual override returns (UnstakeResult memory result) {
    result = WardenHandler.UnstakeResult({
      status: WardenHandler.Status.Failed,
      unstakeTokenAddress: address(0),
      unstakeTokenAmount: 0,
      reinitUnstakeId: 0
    });

    try this.unstake(unstakeId, lpAmount) returns (uint256 withdrawnAmount) {
      result.status = WardenHandler.Status.Success;
      result.reinitUnstakeId = unstakeId;
      result.unstakeTokenAmount = uint128(withdrawnAmount);
      result.unstakeTokenAddress = getUnderlyingToken();
    } catch (bytes memory reason) {
      emit RequestFailed(ActionType.Unstake, unstakeId, reason);
    }
  }

  function _handleReinitRequest() internal virtual override returns (ReinitResult memory) {
    revert('Not supported');
  }
}
