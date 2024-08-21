// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';

import '../libraries/Errors.sol';
import '../interfaces/IWETH9.sol';
import '../interfaces/Lido/ILidoWithdrawalQueue.sol';
import '../interfaces/Lido/IStETH.sol';

/// @title abstract contract implementing the staking interaction with Lido Protocol
abstract contract LidoInteractor is Initializable {
  using SafeERC20 for IERC20;

  event LidoWithdrawStart(uint64 indexed unstakeId, uint256 stEthToWithdraw);
  event LidoWithdrawComplete(uint64 indexed unstakeId, uint256 ethReceived);

  /// @custom:storage-location erc7201:eq-lab.storage.LidoInteractor
  struct LidoInteractorData {
    /// @dev Lido staked ETH token address
    address stETH;
    /// @dev wrapped ETH token address
    address wETH9;
    /// @dev address of Lido WithdrawalQueue contract
    address lidoWithdrawalQueue;
  }

  /// @custom:storage-location erc7201:eq-lab.storage.LidoWithdrawQueue
  struct LidoWithdrawQueue {
    /// @dev queue inner starting index. Increased after popping the first element
    uint128 start;
    /// @dev queue inner ending index. Increased after pushing the new element
    uint128 end;
    /// @dev requestId of withdrawal with given inner index
    mapping(uint128 index => LidoWithdrawQueueElement) items;
  }

  /// @dev struct used in view method to get withdrawal queue element by index
  struct LidoWithdrawQueueElement {
    /// @dev Warden identifier of unstake request
    uint64 unstakeId;
    /// @dev Lido identifier of unstake request
    uint256 requestId;
    /// @dev Requested amount
    uint256 requested;
  }

  /// @dev 'LidoInteractorData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.LidoInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant LidoInteractorDataStorageLocation =
    0x2d1b51a432844d5aca7b5c31b49cc3ff9975ed24b0c347b35bc5ccd56c659300;

  /// @dev 'LidoWithdrawQueue' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.LidoWithdrawQueue")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant LidoWithdrawQueueStorageLocation =
    0x75882ac8a5f1f57df5a632f1fc80cb00a88b7895490864cad38c857f66cad700;

  /// @dev returns storage slot of 'LidoWithdrawQueue' struct
  function _getLidoInteractorDataStorage() internal pure returns (LidoInteractorData storage $) {
    assembly {
      $.slot := LidoInteractorDataStorageLocation
    }
  }

  /// @dev returns storage slot of 'LidoInteractorData' struct
  function _getLidoWithdrawQueueStorage() internal pure returns (LidoWithdrawQueue storage $) {
    assembly {
      $.slot := LidoWithdrawQueueStorageLocation
    }
  }

  /// @notice fallback function for plain native ETH transfers. Allows the one occurring in 'wETH.withdraw()' call only
  receive() external payable {
    LidoInteractorData storage $ = _getLidoInteractorDataStorage();
    if (msg.sender != $.wETH9 && msg.sender != $.lidoWithdrawalQueue) revert Errors.ReceiveValueFail(msg.sender);
  }

  /// @dev initialize method
  /// @param stETH address of Lido staked ETH token
  /// @param wETH9 address of wrapped ETH, which can be used in staking process as an alternative to a native ETH
  function __LidoInteractor_init(address stETH, address wETH9) internal onlyInitializing {
    LidoInteractorData storage $ = _getLidoInteractorDataStorage();
    if (stETH == address(0) || wETH9 == address(0)) revert Errors.ZeroAddress();
    $.stETH = stETH;
    $.wETH9 = wETH9;
  }

  function __LidoInteractor_initV2(address lidoWithdrawalQueue) internal onlyInitializing {
    LidoInteractorData storage $ = _getLidoInteractorDataStorage();
    if (lidoWithdrawalQueue == address(0)) revert Errors.ZeroAddress();
    $.lidoWithdrawalQueue = lidoWithdrawalQueue;
  }

  /// @dev Lido staking method
  /// @param ethAmount amount of either native or wrapped ETH to be staked
  /// @return stEthAmount amount of stETH token received in staking process
  function _lidoStake(uint256 ethAmount) internal returns (uint256 stEthAmount) {
    if (ethAmount == 0) revert Errors.ZeroAmount();
    LidoInteractorData memory data = _getLidoInteractorDataStorage();

    uint256 lidoShares = IStETH(data.stETH).submit{value: ethAmount}(address(0));
    stEthAmount = IStETH(data.stETH).getPooledEthByShares(lidoShares);
  }

  /// @dev starts native eth withdraw from Lido protocol
  /// @param unstakeId unique id of the unstake
  /// @param stEthAmount amount of stEth to perform withdraw on
  function _lidoWithdraw(uint64 unstakeId, uint256 stEthAmount) internal {
    LidoInteractorData storage data = _getLidoInteractorDataStorage();
    ILidoWithdrawalQueue withdrawalQueue = ILidoWithdrawalQueue(data.lidoWithdrawalQueue);

    if (stEthAmount < _getLidoMinWithdrawal()) revert Errors.LowWithdrawalAmount(stEthAmount);

    IERC20(data.stETH).forceApprove(address(withdrawalQueue), stEthAmount);

    uint256 maxLidoWithdrawal = withdrawalQueue.MAX_STETH_WITHDRAWAL_AMOUNT();
    uint256 additionalWithdrawalsNumber = stEthAmount / maxLidoWithdrawal;
    uint256[] memory amounts = new uint256[](additionalWithdrawalsNumber + 1);
    unchecked {
      for (uint256 i; i < additionalWithdrawalsNumber; ++i) {
        amounts[i] = maxLidoWithdrawal;
      }
    }
    amounts[additionalWithdrawalsNumber] = stEthAmount % maxLidoWithdrawal;

    uint256[] memory requestIds = withdrawalQueue.requestWithdrawals(amounts, address(0));
    _enqueue(_getLidoWithdrawQueueStorage(), unstakeId, requestIds, amounts);
    emit LidoWithdrawStart(unstakeId, stEthAmount);
  }

  /// @dev completes if possible the oldest non-fulfilled withdrawal request
  /// @return unstakeId unique id of the unstake
  /// @return ethReceived amount of eth received. Returns 0 if no request was completed
  function _lidoReinit() internal returns (uint64 unstakeId, uint256 ethReceived) {
    LidoWithdrawQueue storage withdrawQueue = _getLidoWithdrawQueueStorage();
    uint128 queueStart = withdrawQueue.start;
    if (queueStart == withdrawQueue.end) return (0, 0);

    LidoWithdrawQueueElement memory withdrawElement = withdrawQueue.items[queueStart];

    try
      ILidoWithdrawalQueue(_getLidoInteractorDataStorage().lidoWithdrawalQueue).claimWithdrawal(
        withdrawElement.requestId
      )
    {
      ethReceived = withdrawElement.requested;
      unstakeId = withdrawElement.unstakeId;
      _dequeue(withdrawQueue);

      emit LidoWithdrawComplete(withdrawElement.unstakeId, ethReceived);
    } catch {
      //TODO: need refactoring: log error in catch or avoid try catch
    }
  }

  /// @dev adds new withdraw request to the end of the LidoWithdrawQueue
  function _enqueue(
    LidoWithdrawQueue storage queue,
    uint64 unstakeId,
    uint256[] memory requestIds,
    uint256[] memory amounts
  ) private {
    uint128 queueEnd = queue.end;
    uint128 length = uint128(requestIds.length);

    unchecked {
      for (uint128 i; i < length; ++i) {
        uint128 index = queueEnd + i;
        queue.items[index] = LidoWithdrawQueueElement({
          requestId: requestIds[i],
          requested: amounts[i],
          unstakeId: unstakeId
        });
      }
      queue.end += length;
    }
  }

  /// @dev pops the first request from the LidoWithdrawQueue after it's fulfilled
  function _dequeue(LidoWithdrawQueue storage queue) private {
    uint128 queueStart = queue.start;
    delete queue.items[queueStart];
    unchecked {
      ++queue.start;
    }
  }

  /// @dev returns LidoWithdrawQueue element by index
  function _getLidoWithdrawalQueueElement(uint128 index) internal view returns (LidoWithdrawQueueElement memory) {
    LidoWithdrawQueue storage queue = _getLidoWithdrawQueueStorage();
    uint128 memoryIndex = queue.start + index;
    if (memoryIndex >= queue.end) revert Errors.NoElementWithIndex(index);

    return queue.items[memoryIndex];
  }

  /// @dev returns min amount allowed to be withdrawn from Lido protocol
  /// @dev can be overridden
  function _getLidoMinWithdrawal() internal view virtual returns (uint256) {
    return ILidoWithdrawalQueue(_getLidoInteractorDataStorage().lidoWithdrawalQueue).MIN_STETH_WITHDRAWAL_AMOUNT();
  }

  /// @notice returns wrapped ETH token address
  function getWeth() public view returns (address) {
    LidoInteractorData storage $ = _getLidoInteractorDataStorage();
    return $.wETH9;
  }
}
