// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.26;

import '../EthYield.sol';

contract TestEthYield is EthYield {
  struct InitStruct {
    //V1
    address stETH;
    address wETH9;
    address elStrategy;
    address elStrategyManager;
    address elDelegationManager;
    address elOperator;
    //V2
    address lidoWithdrawQueue;
    address axelarGateway;
    address axelarGasService;
    string wardenChain;
    string wardenContractAddress;
  }

  function initialize(InitStruct calldata init) external reinitializer(2) {
    //V1
    __Ownable_init(msg.sender);
    __UUPSUpgradeable_init();
    __EigenLayerInteractor_init(
      init.stETH,
      init.elStrategy,
      init.elStrategyManager,
      init.elDelegationManager,
      init.elOperator
    );
    __LidoInteractor_init(init.stETH, init.wETH9);

    //V2
    __LidoInteractor_initV2(init.lidoWithdrawQueue);
    __WardenHandler_init(init.axelarGateway, init.axelarGasService, init.wardenChain, init.wardenContractAddress);
  }
}
