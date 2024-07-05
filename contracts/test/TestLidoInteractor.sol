// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;
import '../interactors/LidoInteractor.sol';

contract TestLidoInteractor is LidoInteractor {
  struct QueueElement {
    uint256 requested;
    uint256 requestId;
  }

  function initialize(
    address weth9,
    address stEth,
    address lidoWithdrawalQueue
  ) external initializer {
    __LidoInteractor_init(stEth, weth9);
    __LidoInteractor_initV2(lidoWithdrawalQueue);
  }

  function stake(uint256 amount) external payable {
    _reinit();
    _lidoStake(amount);
  }

  function withdraw(uint256 amount) external {
    _reinit();
    _lidoWithdraw(amount);
  }

  function getQueueElement(uint256 index) external view returns (QueueElement memory element) {
    LidoWithdrawQueue storage $ = _getLidoWithdrawQueueStorage();
    index += $.start;
    element = QueueElement({requested: $.requested[index], requestId: $.requestId[index]});
  }

  function reinit() external {
    _reinit();
  }

  function _reinit() private {
    _lidoReinit();
  }

  function getQueueStart() external view returns (uint256) {
    return _getLidoWithdrawQueueStorage().start;
  }

  function getQueueEnd() external view returns (uint256) {
    return _getLidoWithdrawQueueStorage().end;
  }

  function getQueueLength() external view returns (uint256) {
    LidoWithdrawQueue storage $ = _getLidoWithdrawQueueStorage();
    return $.end - $.start;
  }
}
