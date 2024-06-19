// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';

import '../interfaces/EigenLayer/IStrategyManager.sol';
import '../interfaces/EigenLayer/IStrategy.sol';
import '../interfaces/Lido/IStETH.sol';

abstract contract EigenLayerInteractor is Initializable {
  using SafeERC20 for IERC20;

  /// @custom:storage-location erc7201:eq-lab.storage.EigenLayerInteractor
  struct EigenLayerInteractorData {
    address stETH;
    address strategy;
    address strategyManager;
  }

  // keccak256(abi.encode(uint256(keccak256("eq-lab.storage.EigenLayerInteractor")) - 1)) & ~bytes32(uint256(0xff))
  bytes32 private constant EigenLayerInteractorDataStorageLocation =
    0xe36167a3404639da86a367e855838355c64e0a9aa7602a57452c5bbf07ac8c00;

  function _getEigenLayerInteractorDataStorage() internal pure returns (EigenLayerInteractorData storage $) {
    assembly {
      $.slot := EigenLayerInteractorDataStorageLocation
    }
  }

  function __EigenLayerInteractor_init(
    address stETH,
    address strategy,
    address strategyManager
  ) internal onlyInitializing {
    require(address(IStrategy(strategy).underlyingToken()) == stETH, 'Wrong strategy or token');

    EigenLayerInteractorData storage $ = _getEigenLayerInteractorDataStorage();
    $.stETH = stETH;
    $.strategy = strategy;
    $.strategyManager = strategyManager;
  }

  function _eigenLayerRestake(uint256 stEthAmount) internal returns (uint256 shares) {
    EigenLayerInteractorData memory data = _getEigenLayerInteractorDataStorage();
    IERC20(data.stETH).approve(data.strategyManager, stEthAmount);
    shares = IStrategyManager(data.strategyManager).depositIntoStrategy(
      IStrategy(data.strategy),
      IERC20(data.stETH),
      stEthAmount
    );
  }
}
