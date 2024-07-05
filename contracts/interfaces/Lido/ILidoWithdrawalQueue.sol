// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

/// @title Interface lido WithdrawalQueue
interface ILidoWithdrawalQueue {
  function requestWithdrawals(
    uint256[] calldata _amounts,
    address _owner
  ) external returns (uint256[] memory requestIds);

  function claimWithdrawal(uint256 _requestId) external;

  function MIN_STETH_WITHDRAWAL_AMOUNT() external view returns (uint256);

  function MAX_STETH_WITHDRAWAL_AMOUNT() external view returns (uint256);
}
