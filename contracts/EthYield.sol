// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';

import './interactors/EigenLayerInteractor.sol';
import './interactors/LidoInteractor.sol';
import './interfaces/IEthYield.sol';
import './YieldStorage.sol';
import './WardenHandler.sol';

contract EthYield is
  UUPSUpgradeable,
  Ownable2StepUpgradeable,
  EigenLayerInteractor,
  LidoInteractor,
  YieldStorage,
  IEthYield,
  WardenHandler
{
  // /// @notice initialize function used during contract deployment
  // /// @param stETH address of a Lido StETH token
  // /// @param wETH9 address of a wrapped ETH
  // /// @param elStrategy address of an EigenLayer strategy (an StEth one specifically in this case)
  // /// @param elStrategyManager address of an EigenLayer strategy manager
  // /// @param elDelegationManager address of an EigenLayer delegation manager
  // /// @param elOperator address of an EigenLayer operator to whom all the restaked stEth will be delegated
  // /// @dev elOperator MUST NOT require any signature, otherwise the initialize tx will revert
  // function initialize(
  //   address stETH,
  //   address wETH9,
  //   address elStrategy,
  //   address elStrategyManager,
  //   address elDelegationManager,
  //   address elOperator
  // ) external initializer {
  //   __Ownable_init(msg.sender);
  //   __UUPSUpgradeable_init();
  //   __EigenLayerInteractor_init(stETH, elStrategy, elStrategyManager, elDelegationManager, elOperator);
  //   __LidoInteractor_init(stETH, wETH9);
  // }

  function initializeV2(
    address lidoWithdrawQueue,
    address axelarGateway,
    address axelarGasService,
    string calldata wardenChain,
    string calldata wardenContractAddress
  ) external reinitializer(2) {
    __LidoInteractor_initV2(lidoWithdrawQueue);
    __WardenHandler_init(axelarGateway, axelarGasService, wardenChain, wardenContractAddress);

    // TODO: add lpAmount totalSupply initial value
  }

  /// @dev method called during the contract upgrade
  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  /// @inheritdoc IEthYield
  function stake(uint256 amount) external virtual returns (uint256 eigenLayerShares) {
    require(msg.sender == address(this));

    uint256 stEthAmount = _lidoStake(amount);
    eigenLayerShares = _eigenLayerRestake(stEthAmount);
    address weth = getWeth();
    _addStake(msg.sender, weth, amount, eigenLayerShares);

    //TODO: add lpAmount calculation
    emit Stake(msg.sender, weth, amount, eigenLayerShares);
  }

  /// @inheritdoc IEthYield
  function unstake(uint64 unstakeId, uint256 eigenLayerSharesAmount) external virtual {
    require(msg.sender == address(this));

    //TODO: add lpAmount calculation
    //TODO: change signature to accept lpAmount

    _eigenLayerWithdraw(unstakeId, eigenLayerSharesAmount);
    // TODO: remove `eigenLayerSharesAmount` from `YieldStorage`
  }

  /// @inheritdoc IEthYield
  /// @dev completes if possible the oldest non-fulfilled withdrawal requests from both EigenLayer and Lido queues
  /// @dev if EigenLayer withdraw was fulfilled, initiates a Lido withdraw for released stEth
  function reinit() external returns (uint64 reinitUnstakeId, uint256 withdrawnAmount) {
    require(msg.sender == address(this));

    (uint64 unstakeId, uint256 stEthWithdrawn) = _eigenLayerReinit();
    if (stEthWithdrawn != 0) {
      _lidoWithdraw(unstakeId, stEthWithdrawn);
    }

    (reinitUnstakeId, withdrawnAmount) = _lidoReinit();
  }

  /// @dev overloads EigenLayer min withdraw amount taking Lido limit into account
  function _getEigenLayerMinSharesToWithdraw() internal view override returns (uint256) {
    return IStrategy(_getEigenLayerInteractorDataStorage().strategy).underlyingToSharesView(_getLidoMinWithdrawal());
  }

  /// @notice returns EigenLayerWithdrawQueueElement element by index
  function getEigenLayerWithdrawalQueueElement(
    uint128 index
  ) external view returns (EigenLayerWithdrawQueueElement memory) {
    return _getEigenLayerWithdrawalQueueElement(index);
  }

  /// @notice returns LidoWithdrawQueue element by index
  function getLidoWithdrawalQueueElement(uint128 index) external view returns (LidoWithdrawQueueElement memory) {
    return _getLidoWithdrawalQueueElement(index);
  }

  /*** WardenHandler ***/

  /// @inheritdoc WardenHandler
  function _handleStakeRequest(
    uint64 stakeId,
    uint256 amountToStake
  ) internal override returns (WardenHandler.StakeResult memory result) {
    ReinitResult memory reinitResult = _handleReinitRequest();

    result = WardenHandler.StakeResult({
      status: WardenHandler.Status.Failed,
      lpAmount: 0,
      unstakeTokenAmount: reinitResult.tokenAmount,
      reinitUnstakeId: reinitResult.reinitUnstakeId
    });

    try this.stake(amountToStake) returns (uint256 lpAmount) {
      result.lpAmount = uint128(lpAmount);
    } catch (bytes memory reason) {
      emit RequestFailed(ActionType.Stake, stakeId, reason);
    }
  }

  /// @inheritdoc WardenHandler
  function _handleUnstakeRequest(
    uint64 unstakeId,
    uint128 lpAmount
  ) internal override returns (WardenHandler.UnstakeResult memory result) {
    ReinitResult memory reinitResult = _handleReinitRequest();

    result = WardenHandler.UnstakeResult({
      status: WardenHandler.Status.Failed,
      unstakeTokenAddress: reinitResult.tokenAddress,
      unstakeTokenAmount: reinitResult.tokenAmount,
      reinitUnstakeId: reinitResult.reinitUnstakeId
    });

    try this.unstake(unstakeId, lpAmount) {
      result.status = WardenHandler.Status.Success;
    } catch (bytes memory reason) {
      emit RequestFailed(ActionType.Unstake, unstakeId, reason);
    }
  }

  /// @inheritdoc WardenHandler
  function _handleReinitRequest() internal override returns (ReinitResult memory result) {
    result = WardenHandler.ReinitResult({tokenAddress: getWeth(), tokenAmount: 0, reinitUnstakeId: 0});

    try this.reinit() returns (uint64 reinitUnstakeId, uint256 withdrawnAmount) {
      result.reinitUnstakeId = reinitUnstakeId;
      result.tokenAmount = uint128(withdrawnAmount);
    } catch (bytes memory reason) {
      emit RequestFailed(ActionType.Reinit, 0, reason);
    }
  }
}
