// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

/// @title Interface lido WithdrawalQueue
/// @dev https://github.com/lidofinance/lido-dao/blob/5fcedc6e9a9f3ec154e69cff47c2b9e25503a78a/contracts/0.8.9/WithdrawalQueue.sol
interface ILidoWithdrawalQueue {
  function requestWithdrawals(
    uint256[] calldata _amounts,
    address _owner
  ) external returns (uint256[] memory requestIds);

  function claimWithdrawal(uint256 _requestId) external;

  function MIN_STETH_WITHDRAWAL_AMOUNT() external view returns (uint256);

  function MAX_STETH_WITHDRAWAL_AMOUNT() external view returns (uint256);
}
