// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';

import '../interfaces/IWETH9.sol';
import '../interfaces/Lido/IWstETH.sol';
import '../interfaces/Lido/IStETH.sol';

abstract contract LidoInteractor is Initializable {
  using SafeERC20 for IERC20;

  /// @custom:storage-location erc7201:eq-lab.storage.LidoInteractor
  struct LidoInteractorData {
    address stETH;
    address wstETH;
    address wETH9;
  }

  // keccak256(abi.encode(uint256(keccak256("eq-lab.storage.LidoInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant LidoInteractorDataStorageLocation =
    0x2d1b51a432844d5aca7b5c31b49cc3ff9975ed24b0c347b35bc5ccd56c659300;

  function _getLidoInteractorDataStorage() internal pure returns (LidoInteractorData storage $) {
    assembly {
      $.slot := LidoInteractorDataStorageLocation
    }
  }

  receive() external payable {
    require(msg.sender == _getLidoInteractorDataStorage().wETH9, 'Not WETH9');
  }

  function __LidoInteractor_init(address stETH, address wstETH, address wETH9) internal onlyInitializing {
    LidoInteractorData storage $ = _getLidoInteractorDataStorage();
    $.stETH = stETH;
    $.wstETH = wstETH;
    $.wETH9 = wETH9;
  }

  function _stake(uint256 ethAmount) internal returns (uint256 wstEthStakedAmount) {
    LidoInteractorData memory data = _getLidoInteractorDataStorage();

    if (msg.value == 0) {
      IERC20(data.wETH9).safeTransferFrom(msg.sender, address(this), ethAmount);
      IWETH9(data.wETH9).withdraw(ethAmount);
    } else if (msg.value != ethAmount) {
      revert('Wrong msg.value');
    }

    // call lido, get stEth
    uint256 stEthStakedAmount = IStETH(data.stETH).submit{value: ethAmount}(address(0));

    // wrap stEth to wsthEth
    IERC20(data.stETH).approve(data.wstETH, stEthStakedAmount);
    wstEthStakedAmount = IWstETH(data.wstETH).wrap(stEthStakedAmount);
  }
}
