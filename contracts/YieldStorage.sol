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
    /// @dev not used since v2
    mapping(address /* token */ => uint256) _totalStakedAmount;
    /// @dev not used since v2
    mapping(address /* token */ => uint256) _totalShares;
    /// @dev not used since v2
    mapping(address /* token */ => mapping(address /* user */ => uint256)) _stakedAmount;
    /// @dev not used since v2
    mapping(address /* token */ => mapping(address /* user */ => uint256)) _shares;
    /// @dev total amount of shares received from underlying protocol
    uint256 totalShares;
    /// @dev total amount of lp tokens minted
    uint256 totalLpt;
  }

  /// @custom:storage-location erc7201:eq-lab.storage.UserWardenData
  /// @dev not used since v2
  struct UserWardenData {
    mapping(address /* evmAddress */ => string) _wardenAddress;
  }

  /// @dev 'StakingData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.StakingData")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant StakingDataStorageLocation =
    0x69b3bfac4ac6bf246ceef7427e431f481bd6bde26467dffa51aa8b49ac672600;

  function __YieldStorage_initV2(address token) internal onlyInitializing {
    StakingData storage $ = _getStakingDataStorage();

    $.totalShares = $._totalShares[token];
    $.totalLpt = $._totalStakedAmount[token];
  }

  /// @notice returns pointer to the storage slot of StakingData struct
  function _getStakingDataStorage() internal pure returns (StakingData storage $) {
    assembly {
      $.slot := StakingDataStorageLocation
    }
  }

  /// @notice adds new staking amounts to storage
  /// @param shares amount of shares that were received from the staking protocol
  function _addStake(uint256 shares, uint256 underlyingAmount) internal returns (uint256 lpAmount) {
    StakingData storage $ = _getStakingDataStorage();
    lpAmount = $.totalShares == 0 ? underlyingAmount : _sharesToLpAmount(shares);
    $.totalLpt += lpAmount;
    $.totalShares += shares;
  }

  /// @notice sets user staking data to zero and decreases total values
  function _removeStake(uint256 lpTokenAmount) internal returns (uint256 sharesAmount) {
    StakingData storage $ = _getStakingDataStorage();
    sharesAmount = _lpAmountToShares(lpTokenAmount);
    $.totalShares -= sharesAmount;
    $.totalLpt -= lpTokenAmount;
  }

  /// @dev Convert shares to lp amount
  function _sharesToLpAmount(uint256 sharesAmount) internal view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return ($.totalLpt * sharesAmount) / $.totalShares;
  }

  /// @dev Convert lp amount to shares
  function _lpAmountToShares(uint256 lpAmount) internal view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return ($.totalShares * lpAmount) / $.totalLpt;
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
