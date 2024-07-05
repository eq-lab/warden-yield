// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../interfaces/Lido/ILidoWithdrawalQueue.sol';

interface ILidoWithdrawalQueueExtended is ILidoWithdrawalQueue {
  function finalize(uint256 _lastIdToFinalize, uint256 _maxShareRate) external payable;

  function FINALIZE_ROLE() external view returns (bytes32);

  function getRoleMember(bytes32 role, uint256 index) external view returns (address);
}
