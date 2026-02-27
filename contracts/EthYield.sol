// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol';
import '@openzeppelin/contracts-upgradeable/access/Ownable2StepUpgradeable.sol';
import '@openzeppelin/contracts/utils/math/Math.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';

import './interactors/EigenLayerInteractor.sol';
import './interactors/LidoInteractor.sol';
import './interfaces/IEthYield.sol';
import './YieldStorage.sol';

contract EthYield is UUPSUpgradeable, Ownable2StepUpgradeable, EigenLayerInteractor, LidoInteractor, YieldStorage {
  bool public userWithdrawalsActive;

  event Withdrawn(address indexed user, uint256 amount);

  /// @notice initialize function used during contract upgrade and deployment
  /// @dev This method goes separate from the `initialize` during deployment to avoid contract size limitation
  /// @param lidoWithdrawQueue address of lido withdrawal queue
  function initializeV2(address lidoWithdrawQueue) external reinitializer(2) {
    __LidoInteractor_initV2(lidoWithdrawQueue);

    _eigenLayerFullWithdraw();
  }

  /// @dev method called during the contract upgrade
  function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

  function startLidoWithdrawal() external {
    _eigenLayerCompleteWithdrawal();
    _lidoFullWithdraw();
  }

  function completeLidoWithdrawal() external {
    _lidoCompleteWithdrawal();

    if (!hasPendingLidoWithdrawal()) {
      userWithdrawalsActive = true;
      IWETH9(getWeth()).deposit{value: address(this).balance}();
    }
  }

  function withdrawAll() external {
    if (!userWithdrawalsActive) revert Errors.Forbidden();

    address weth = getWeth();

    uint256 userShares = userShares(msg.sender, weth);
    if (userShares == 0) revert Errors.ZeroAmount();

    uint256 totalShares = totalShares(weth);
    uint256 wethBalance = IWETH9(weth).balanceOf(address(this));

    uint256 transferAmount = totalShares == userShares
      ? wethBalance
      : Math.mulDiv(wethBalance, userShares, totalShares);
    SafeERC20.safeTransfer(IERC20(weth), msg.sender, transferAmount);
    _removeStake(msg.sender, weth);

    emit Withdrawn(msg.sender, transferAmount);
  }
}
