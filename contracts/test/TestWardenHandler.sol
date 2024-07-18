// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../WardenHandler.sol';

contract TestWardenHandler is WardenHandler {
  WardenHandler.StakeResult public _stakeResult;
  WardenHandler.UnstakeResult public _unstakeResult;
  WardenHandler.ReinitResult public _reinitResult;

  function initialize(
    address axelarGateway,
    address axelarGasService,
    string calldata wardenChain,
    string calldata wardenContractAddress
  ) external initializer {
    __WardenHandler_init(axelarGateway, axelarGasService, wardenChain, wardenContractAddress);
  }

  function setStakeResult(WardenHandler.StakeResult calldata stakeResult) external {
    _stakeResult = stakeResult;
  }

  function setUnstakeResult(WardenHandler.UnstakeResult calldata unstakeResult) external {
    _unstakeResult = unstakeResult;
  }

  function setReinitResult(WardenHandler.ReinitResult calldata reinitResult) external {
    _reinitResult = reinitResult;
  }

  function _handleStakeRequest(uint64, address, uint256) internal view override returns (StakeResult memory) {
    return _stakeResult;
  }

  function _handleUnstakeRequest(uint64, uint128) internal view override returns (UnstakeResult memory) {
    return _unstakeResult;
  }

  function _handleReinitRequest() internal view override returns (ReinitResult memory) {
    return _reinitResult;
  }
}
