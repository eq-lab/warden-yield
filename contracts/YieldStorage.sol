// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/utils/Strings.sol';

import './libraries/Errors.sol';

abstract contract YieldStorage {
  using Strings for string;

  /// @custom:storage-location erc7201:eq-lab.storage.StakingData
  struct StakingData {
    mapping(address /* token */ => uint256) _totalStakedAmount;
    mapping(address /* token */ => uint256) _totalShares;
    mapping(address /* token */ => mapping(address => uint256)) _stakedAmount;
    mapping(address /* token */ => mapping(address => uint256)) _shares;
  }

  /// @custom:storage-location erc7201:eq-lab.storage.UserWardenData
  struct UserWardenData {
    mapping(address /* evmAddress */ => string) _wardenAddress;
  }

  // keccak256(abi.encode(uint256(keccak256("eq-lab.storage.StakingData")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant StakingDataStorageLocation =
    0x69b3bfac4ac6bf246ceef7427e431f481bd6bde26467dffa51aa8b49ac672600;

  // keccak256(abi.encode(uint256(keccak256("eq-lab.storage.UserWardenData")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant UserWardenDataStorageLocation =
    0x730e2444c52438fad4871347d49b7d0d3be1e25c4cf433da8143e5ca5ac50700;

  function _getStakingDataStorage() internal pure returns (StakingData storage $) {
    assembly {
      $.slot := StakingDataStorageLocation
    }
  }

  function _getUserWardenDataStorage() internal pure returns (UserWardenData storage $) {
    assembly {
      $.slot := UserWardenDataStorageLocation
    }
  }

  function _addStake(address user, address token, uint256 stakeAmount, uint256 shares) internal {
    StakingData storage $ = _getStakingDataStorage();
    $._stakedAmount[token][user] += stakeAmount;
    $._totalStakedAmount[token] += stakeAmount;

    $._shares[token][user] += shares;
    $._totalShares[token] += shares;
  }

  function _removeStake(address user, address token) internal {
    StakingData storage $ = _getStakingDataStorage();
    $._totalStakedAmount[token] -= $._stakedAmount[token][user];
    $._stakedAmount[token][user] = 0;

    $._totalShares[token] -= $._shares[token][user];
    $._shares[token][user] = 0;
  }

  function _addWardenAddress(address user, string calldata userWardenAddress) internal {
    UserWardenData storage $ = _getUserWardenDataStorage();
    string memory currentWardenAddress = $._wardenAddress[user];

    if (bytes(currentWardenAddress).length == 0) {
      $._wardenAddress[user] = userWardenAddress;
    } else if (!currentWardenAddress.equal(userWardenAddress)) {
      revert Errors.WrongWardenAddress(user, currentWardenAddress, userWardenAddress);
    }
  }

  function totalShares(address token) public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._totalShares[token];
  }

  function totalStakedAmount(address token) public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._totalStakedAmount[token];
  }

  function userShares(address user, address token) public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._shares[token][user];
  }

  function userStakedAmount(address user, address token) public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._stakedAmount[token][user];
  }

  function wardenAddress(address user) public view returns (string memory) {
    UserWardenData storage $ = _getUserWardenDataStorage();
    return $._wardenAddress[user];
  }
}
