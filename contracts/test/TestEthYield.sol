// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../EthYield.sol';

///@dev Version of EthYield contract that holds all initializer functions
contract TestEthYield is EthYield {
  function initialize(
    address stETH,
    address wETH9,
    address elStrategy,
    address elStrategyManager,
    address elDelegationManager,
    address elOperator
  ) external initializer {
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __EigenLayerInteractor_init(stETH, elStrategy, elStrategyManager, elDelegationManager, elOperator);
    __LidoInteractor_init(stETH, wETH9);
  }
}
