import { Signer } from 'ethers';
import { AaveYield__factory, EthYield__factory } from '../../typechain-types';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { createDefaultBaseState, DeployState, getStateFileName, StateFile, StateStore } from '../state-store';
import { SimpleLogger } from '../logger';
import path from 'path';
import { createDefaultBaseDeployment, DeploymentFile, DeploymentState, DeploymentStore } from '../deployment-store';
import { AaveYieldUpgradeConfig, EthYieldUpgradeConfig, UpgradeConfig } from './config';
import { EthConnectionConfig } from '../config-common';

export async function upgradeWardenYield(
  signer: Signer,
  config: UpgradeConfig,
  network: string,
  dryRun: boolean,
  hre: HardhatRuntimeEnvironment
): Promise<void> {
  const statesDirName = 'states';
  const stateFileName = getStateFileName(network, statesDirName);
  const actualStateFile = path.join(__dirname, `data`, `configs`, network, stateFileName);
  const actualDeploymentFile = path.join(__dirname, `data`, `contracts`, `${network}.json`);

  const logger = new SimpleLogger((x) => console.error(x));
  const stateStore = new StateFile(
    'WardenYield',
    createDefaultBaseState,
    actualStateFile,
    !dryRun,
    logger
  ).createStateStore();

  const deploymentStore = new DeploymentFile(
    'WardenYield',
    createDefaultBaseDeployment,
    actualDeploymentFile,
    !dryRun,
    logger
  ).createDeploymentStore();


  if (config.ethYield !== undefined) {
    await upgradeEthYield(signer, config.ethYield, config.ethConnection, hre, stateStore, deploymentStore);
  }

  if (config.aaveYield !== undefined) {
    await upgradeAaveYield(signer, config.aaveYield, config.ethConnection, hre, stateStore, deploymentStore);
  }
}

async function upgradeEthYield(
  signer: Signer,
  ethConfig: EthYieldUpgradeConfig,
  ethConnectionConfig: EthConnectionConfig,
  hre: HardhatRuntimeEnvironment,
  stateStore: StateStore,
  deploymentStore: DeploymentStore
) {
  const ethYieldDeployment = deploymentStore.getById('ethYield-proxy');
  if (ethYieldDeployment === undefined) {
    throw new Error(`EthYield deployment wasn't found: nothing to upgrade`);
  }
  const ethYieldProxyAddress = ethYieldDeployment.address;

  await hre.upgrades.upgradeProxy(ethYieldProxyAddress, new EthYield__factory().connect(signer), {
    call: {
      fn: 'initializeV2',
      args: [
        ethConfig.lidoWithdrawalQueue,
        ethConfig.wardenHandler.axelarGateway,
        ethConfig.wardenHandler.axelarGasService,
        ethConfig.wardenHandler.wardenChain,
        ethConfig.wardenHandler.wardenContractAddress,
      ],
    },
    txOverrides: {
      gasLimit: ethConnectionConfig.ethOptions.gasLimit,
      gasPrice: ethConnectionConfig.ethOptions.gasPrice,
    },
  });

  const implementationAddress = await hre.upgrades.erc1967.getImplementationAddress(ethYieldProxyAddress);
  console.log(`EthYield proxy: ${ethYieldProxyAddress}, implementation: ${implementationAddress}`);


  stateStore.setById('ethYield-proxy', <DeployState>{ address: ethYieldProxyAddress });
  stateStore.setById('ethYield-impl', <DeployState>{ address: implementationAddress });
  deploymentStore.setById('ethYield', <DeploymentState>{
    address: ethYieldProxyAddress,
    implementation: implementationAddress,
  });
}

async function upgradeAaveYield(
  signer: Signer,
  aaveConfig: AaveYieldUpgradeConfig,
  ethConnectionConfig: EthConnectionConfig,
  hre: HardhatRuntimeEnvironment,
  stateStore: StateStore,
  deploymentStore: DeploymentStore
) {
  const aaveYieldDeployment = deploymentStore.getById('aaveYield-proxy');
  if (aaveYieldDeployment === undefined) {
    throw new Error(`AaveYield deployment wasn't found: nothing to upgrade`);
  }
  const aaveYieldProxyAddress = aaveYieldDeployment.address;

  await hre.upgrades.upgradeProxy(aaveYieldProxyAddress, new AaveYield__factory().connect(signer), {
    call: {
      fn: 'initializeV2',
      args: [
        aaveConfig.underlyingToken,
        aaveConfig.wardenHandler.axelarGateway,
        aaveConfig.wardenHandler.axelarGasService,
        aaveConfig.wardenHandler.wardenChain,
        aaveConfig.wardenHandler.wardenContractAddress,
      ],
    },
    txOverrides: {
      gasLimit: ethConnectionConfig.ethOptions.gasLimit,
      gasPrice: ethConnectionConfig.ethOptions.gasPrice,
    },
  });

  const implementationAddress = await hre.upgrades.erc1967.getImplementationAddress(aaveYieldProxyAddress);
  console.log(`AaveYield proxy: ${aaveYieldProxyAddress}, implementation: ${implementationAddress}`);


  stateStore.setById('aaveYield-proxy', <DeployState>{ address: aaveYieldProxyAddress });
  stateStore.setById('aaveYield-impl', <DeployState>{ address: implementationAddress });
  deploymentStore.setById('aaveYield', <DeploymentState>{
    address: aaveYieldProxyAddress,
    implementation: implementationAddress,
  });
}
