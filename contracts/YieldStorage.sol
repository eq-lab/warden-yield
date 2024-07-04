// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/utils/Strings.sol';

import './libraries/Errors.sol';

/// @notice abstract contract for storing data related to staking process
abstract contract YieldStorage {
  using Strings for string;

  /// @custom:storage-location erc7201:eq-lab.storage.StakingData
  struct StakingData {
    /// @dev total amount of 'token' locked by users via this contract
    mapping(address /* token */ => uint256) _totalStakedAmount;
    /// @dev total amount shares received by users after locking 'token'
    mapping(address /* token */ => uint256) _totalShares;
    /// @dev total amount of 'token' locked by the 'user' via this contract
    mapping(address /* token */ => mapping(address /* user */ => uint256)) _stakedAmount;
    /// @dev total amount shares received by 'user' after locking the 'token'
    mapping(address /* token */ => mapping(address /* user */ => uint256)) _shares;
  }

  /// @custom:storage-location erc7201:eq-lab.storage.UserWardenData
  struct UserWardenData {
    /// @dev mapping of evm -> warden user addresses
    mapping(address /* evmAddress */ => string) _wardenAddress;
  }

  event WardenAddressSet(address indexed evmAddress, string indexed wardenAddress);

  /// @dev 'StakingData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.StakingData")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant StakingDataStorageLocation =
    0x69b3bfac4ac6bf246ceef7427e431f481bd6bde26467dffa51aa8b49ac672600;

  /// @dev 'UserWardenData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.UserWardenData")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant UserWardenDataStorageLocation =
    0x730e2444c52438fad4871347d49b7d0d3be1e25c4cf433da8143e5ca5ac50700;

  /// @notice returns pointer to the storage slot of StakingData struct
  function _getStakingDataStorage() internal pure returns (StakingData storage $) {
    assembly {
      $.slot := StakingDataStorageLocation
    }
  }

  /// @notice returns pointer to the storage slot of UserWardenData struct
  function _getUserWardenDataStorage() internal pure returns (UserWardenData storage $) {
    assembly {
      $.slot := UserWardenDataStorageLocation
    }
  }

  /// @notice adds new staking amounts to storage
  /// @param user address of a user whose staking position increases
  /// @param token address of a token which was tranferred to the staking protocol (e.g. weth for Lido)
  /// @param stakeAmount amount of token that was transferred to the staking protocol
  /// @param shares amount of shares that were received from the staking protocol
  function _addStake(address user, address token, uint256 stakeAmount, uint256 shares) internal {
    StakingData storage $ = _getStakingDataStorage();
    $._stakedAmount[token][user] += stakeAmount;
    $._totalStakedAmount[token] += stakeAmount;

    $._shares[token][user] += shares;
    $._totalShares[token] += shares;
  }

  /// @notice sets user staking data to zero and decreases total values
  /// @param user address of a user whose staking position is removed
  /// @param token address of a token which was used in the staking protocol (e.g. weth for Lido)
  function _removeStake(address user, address token) internal {
    StakingData storage $ = _getStakingDataStorage();
    $._totalStakedAmount[token] -= $._stakedAmount[token][user];
    $._stakedAmount[token][user] = 0;

    $._totalShares[token] -= $._shares[token][user];
    $._shares[token][user] = 0;
  }

  /// @notice sets user's warden address.
  /// @dev the warden address can't be changed if set
  /// @param user eth address
  /// @param userWardenAddress user address in Warden blockchain
  function _addWardenAddress(address user, string calldata userWardenAddress) internal {
    UserWardenData storage $ = _getUserWardenDataStorage();
    string memory currentWardenAddress = $._wardenAddress[user];

    if (bytes(currentWardenAddress).length == 0) {
      $._wardenAddress[user] = userWardenAddress;
      emit WardenAddressSet(user, userWardenAddress);
    } else if (!currentWardenAddress.equal(userWardenAddress)) {
      revert Errors.WrongWardenAddress(user, currentWardenAddress, userWardenAddress);
    }
  }

  /// @notice returns total shares recieved in all stake calls
  function totalShares(address token) external view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._totalShares[token];
  }

  /// @notice returns total amount of tokens transfered to staking protocols in stake calls
  function totalStakedAmount(address token) external view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._totalStakedAmount[token];
  }

  /// @notice returns total shares recieved in all stake calls by user
  /// @param user user address whose shares are returned
  function userShares(address user, address token) public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._shares[token][user];
  }

  /// @notice returns total amount of tokens transfered to staking protocols in stake calls by user
  /// @param user user address whose staked tokens are returned
  function userStakedAmount(address user, address token) external view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._stakedAmount[token][user];
  }

  /// @notice returns user warden address.
  /// @param user eth address
  function wardenAddress(address user) external view returns (string memory) {
    UserWardenData storage $ = _getUserWardenDataStorage();
    return $._wardenAddress[user];
  }
}
