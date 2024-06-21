// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

abstract contract YieldStorage {
  /// @custom:storage-location erc7201:eq-lab.storage.StakingData
  struct StakingData {
    mapping(address /* token */ => uint256) _totalStakedAmount;
    mapping(address /* token */ => uint256) _totalShares;
    mapping(address /* token */ => mapping(address => uint256)) _stakedAmount;
    mapping(address /* token */ => mapping(address => uint256)) _shares;
    mapping(address /* evmAddress */ => string) _wardenAddress;
  }

  // keccak256(abi.encode(uint256(keccak256("eq-lab.storage.StakingData")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant StakingDataStorageLocation =
    0x69b3bfac4ac6bf246ceef7427e431f481bd6bde26467dffa51aa8b49ac672600;

  function _getStakingDataStorage() internal pure returns (StakingData storage $) {
    assembly {
      $.slot := StakingDataStorageLocation
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
    StakingData storage $ = _getStakingDataStorage();
    return $._wardenAddress[user];
  }
}
