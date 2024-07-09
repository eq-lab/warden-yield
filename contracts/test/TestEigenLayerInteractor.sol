// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../interactors/EigenLayerInteractor.sol';
import '../interactors/LidoInteractor.sol';

contract TestEigenLayerInteractor is EigenLayerInteractor, LidoInteractor {
  using SafeERC20 for IERC20;

  struct QueueElement {
    uint256 shares;
    uint32 blockNumber;
  }

  event Stake(uint256 eigenLayerShares);

  function initialize(
    address weth9,
    address stEth,
    address strategy,
    address strategyManager,
    address delegationManager,
    address operator
  ) external initializer {
    __EigenLayerInteractor_init(stEth, strategy, strategyManager, delegationManager, operator);
    __LidoInteractor_init(stEth, weth9);
  }

  function stake(uint256 amount) external payable {
    _eigenLayerReinit();
    uint256 stEthAmount = _lidoStake(amount);
    uint256 eigenLayerShares = _eigenLayerRestake(stEthAmount);
    emit Stake(eigenLayerShares);
  }

  function withdraw(uint256 amount) external {
    _eigenLayerReinit();
    _eigenLayerWithdraw(amount);
  }

  function getQueueElement(uint256 index) external view returns (QueueElement memory element) {
    EigenLayerWithdrawQueue storage $ = _getEigenLayerWithdrawQueueStorage();
    index += $.start;
    element = QueueElement({shares: $.shares[index], blockNumber: $.blockNumber[index]});
  }

  function reinit() external {
    _eigenLayerReinit();
  }

  function getQueueStart() external view returns (uint256) {
    return _getEigenLayerWithdrawQueueStorage().start;
  }

  function getQueueEnd() external view returns (uint256) {
    return _getEigenLayerWithdrawQueueStorage().end;
  }

  function getQueueLength() external view returns (uint256) {
    EigenLayerWithdrawQueue storage $ = _getEigenLayerWithdrawQueueStorage();
    return $.end - $.start;
  }

  function _getEigenLayerMinSharesToWithdraw() internal pure override returns(uint256) {
    return 0;
  }
}
