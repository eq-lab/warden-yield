// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts/utils/math/Math.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';

import '../libraries/Errors.sol';
import '../interfaces/IWETH9.sol';
import '../interfaces/Lido/IStETH.sol';

/// @title abstract contract implementing the staking interaction with Lido Protocol
abstract contract LidoInteractor is Initializable {
  using SafeERC20 for IERC20;

  /// @custom:storage-location erc7201:eq-lab.storage.LidoInteractor
  struct LidoInteractorData {
    /// @dev Lido staked ETH token address
    address stETH;
    /// @dev wrapped ETH token address
    address wETH9;
  }

  /// @dev 'LidoInteractorData' storage slot address
  /// @dev keccak256(abi.encode(uint256(keccak256("eq-lab.storage.LidoInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant LidoInteractorDataStorageLocation =
    0x2d1b51a432844d5aca7b5c31b49cc3ff9975ed24b0c347b35bc5ccd56c659300;

  /// @dev returns storage slot of 'LidoInteractorData' struct
  function _getLidoInteractorDataStorage() internal pure returns (LidoInteractorData storage $) {
    assembly {
      $.slot := LidoInteractorDataStorageLocation
    }
  }

  /// @notice fallback function for plain native ETH transfers. Allows the one occuring in 'wETH.withdraw()' call only
  receive() external payable {
    if (msg.sender != _getLidoInteractorDataStorage().wETH9) revert Errors.NotWETH9(msg.sender);
  }

  /// @dev initialize method
  /// @param stETH address of Lido staked ETH token
  /// @param wETH9 address of wrapped ETH, which can be used is staking process as an alternative to a native ETH
  function __LidoInteractor_init(address stETH, address wETH9) internal onlyInitializing {
    LidoInteractorData storage $ = _getLidoInteractorDataStorage();
    if (stETH == address(0) || wETH9 == address(0)) revert Errors.ZeroAddress();
    $.stETH = stETH;
    $.wETH9 = wETH9;
  }

  /// @dev Lido staking method
  /// @param ethAmount amount of either native or wrapped ETH to be staked
  /// @return stEthAmount amount of stETH token received in staking process
  function _lidoStake(uint256 ethAmount) internal returns (uint256 stEthAmount) {
    if (ethAmount == 0) revert Errors.ZeroAmount();
    LidoInteractorData memory data = _getLidoInteractorDataStorage();

    if (msg.value == 0) {
      IERC20(data.wETH9).safeTransferFrom(msg.sender, address(this), ethAmount);
      IWETH9(data.wETH9).withdraw(ethAmount);
    } else if (msg.value != ethAmount) {
      revert Errors.WrongMsgValue(msg.value, ethAmount);
    }

    uint256 lidoShares = IStETH(data.stETH).submit{value: ethAmount}(address(0));
    stEthAmount = IStETH(data.stETH).getPooledEthByShares(lidoShares);
  }

  /// @notice returns wrapped ETH token address
  function getWeth() public view returns (address) {
    LidoInteractorData storage $ = _getLidoInteractorDataStorage();
    return $.wETH9;
  }
}
