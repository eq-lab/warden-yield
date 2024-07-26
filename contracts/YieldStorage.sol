// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/utils/Strings.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';

import './libraries/Errors.sol';

/// @notice abstract contract for storing data related to staking process
abstract contract YieldStorage is Initializable {
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
    uint256 totalShares;
    uint256 totalLpt;
  }

  /// @custom:storage-location erc7201:eq-lab.storage.UserWardenData
  struct UserWardenData {
    /// @dev mapping of evm -> warden user addresses
    mapping(address /* evmAddress */ => string) _wardenAddress;
  }

  /// @dev 'StakingData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.StakingData")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant StakingDataStorageLocation =
    0x69b3bfac4ac6bf246ceef7427e431f481bd6bde26467dffa51aa8b49ac672600;

  function __YieldStorage_initV2(address token) internal onlyInitializing {
    StakingData storage $ = _getStakingDataStorage();

    $.totalShares = $._totalShares[token];
  }

  /// @notice returns pointer to the storage slot of StakingData struct
  function _getStakingDataStorage() internal pure returns (StakingData storage $) {
    assembly {
      $.slot := StakingDataStorageLocation
    }
  }

  /// @notice adds new staking amounts to storage
  /// @param shares amount of shares that were received from the staking protocol
  function _addStake(uint256 shares, uint256 lpTokenAmount) internal {
    StakingData storage $ = _getStakingDataStorage();
    $.totalShares += shares;
    $.totalLpt += lpTokenAmount;
  }

  /// @notice sets user staking data to zero and decreases total values
  /// @param shares amount of shares that were released from the staking protocol
  function _removeStake(uint256 shares, uint256 lpTokenAmount) internal {
    StakingData storage $ = _getStakingDataStorage();
    $.totalShares -= shares;
    $.totalLpt -= lpTokenAmount;
  }

  /// @notice returns total shares received in all stake calls
  function totalShares() external view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $.totalShares;
  }

  /// @notice returns total lp tokens amount
  function totalLpTokens() external view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $.totalLpt;
  }
}
