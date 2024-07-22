// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../AaveYield.sol';

///@dev Version of EthYield contract that holds all initializer functions
contract TestAaveYield is AaveYield {
  /// @notice initialize function used during contract deployment
  /// @param aavePool address of a Aave pool
  /// @param tokens array with addresses of tokens which will be used in the Aave pool
  function initialize(address aavePool, address[] calldata tokens) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __AaveInteractor_init(aavePool, tokens);
  }
}
