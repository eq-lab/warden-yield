// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

contract YieldStorage {
  /// @custom:storage-location erc7201:eq-lab.storage.StakingData
  struct StakingData {
    uint256 _totalStakedAmount;
    uint256 _totalInputAmount;
    mapping(address => uint256) _inputAmount;
    mapping(address => uint256) _stakedAmount;
    mapping(address => string) _wardenAddress;
  }

  // keccak256(abi.encode(uint256(keccak256("eq-lab.storage.StakingData")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant StakingDataStorageLocation = 0x69b3bfac4ac6bf246ceef7427e431f481bd6bde26467dffa51aa8b49ac672600;

  function _getStakingDataStorage() internal pure returns (StakingData storage $) {
    assembly {
      $.slot := StakingDataStorageLocation
    }
  }

  function totalInputAmount() public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._totalInputAmount;
  }

  function totalStakedAmount() public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._totalStakedAmount;
  }

  function inputAmount(address user) public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._inputAmount[user];
  }

  function stakedAmount(address user) public view returns (uint256) {
    StakingData storage $ = _getStakingDataStorage();
    return $._stakedAmount[user];
  }

  function wardenAddress(address user) public view returns (string memory) {
    StakingData storage $ = _getStakingDataStorage();
    return $._wardenAddress[user];
  }
}
