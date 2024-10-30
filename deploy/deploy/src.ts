import { Signer } from 'ethers';
import { AaveYieldDeploymentConfig, DeployConfig, EthYieldDeploymentConfig } from './config';
import { AaveYield, AaveYield__factory, EthYield, EthYield__factory } from '../../typechain-types';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { createDefaultBaseState, DeployState, getStateFileName, StateFile, StateStore } from '../state-store';
import { SimpleLogger } from '../logger';
import path from 'path';
import { createDefaultBaseDeployment, DeploymentFile, DeploymentState, DeploymentStore } from '../deployment-store';
import { EthConnectionConfig, EthOptions } from '../config-common';
import { getMaxFeePerGas } from '../common';

export async function deployWardenYield(
  signer: Signer,
  config: DeployConfig,
  network: string,
  dryRun: boolean,
  hre: HardhatRuntimeEnvironment
): Promise<void> {
  const statesDirName = 'states';
  const stateFileName = getStateFileName(network, statesDirName);
  const actualStateFile = path.join(path.parse(__dirname).dir, `data`, `configs`, network, stateFileName);
  const actualDeploymentFile = path.join(path.parse(__dirname).dir, `data`, `contracts`, `${network}.json`);

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

  if (config.aaveYield !== undefined) {
    await deployAaveYield(signer, config.aaveYield, config.ethConnection.ethOptions, hre, stateStore, deploymentStore);
  }
  if (config.ethYield !== undefined) {
    await deployEthYield(signer, config.ethYield, config.ethConnection.ethOptions, hre, stateStore, deploymentStore);
  }

  console.log(`State file: \n${stateStore.stringify()}`);
  console.log(`Deployment file: \n${deploymentStore.stringify()}`);
}

async function deployAaveYield(
  signer: Signer,
  aaveConfig: AaveYieldDeploymentConfig,
  ethOptions: EthOptions,
  hre: HardhatRuntimeEnvironment,
  stateStore: StateStore,
  deploymentStore: DeploymentStore
): Promise<void> {
  console.log(
    `Deploy AaveYield. AavePoolProvider: ${aaveConfig.aavePoolProvider}, underlyingToken: [${aaveConfig.underlyingToken}]`
  );

  const state = stateStore.getById('aaveYield-proxy');
  let aaveYieldAddress: string;
  if (state !== undefined) {
    console.log(`AaveYield already deployed. Skip.`);
    aaveYieldAddress = state.address;
  } else {
    const aaveYield = (await hre.upgrades.deployProxy(
      new AaveYield__factory().connect(signer),
      [
        aaveConfig.aavePoolProvider,
        aaveConfig.underlyingToken,
        aaveConfig.wardenHandler.axelarGateway,
        aaveConfig.wardenHandler.axelarGasService,
        aaveConfig.wardenHandler.evmChainName,
        aaveConfig.wardenHandler.wardenChain,
        aaveConfig.wardenHandler.wardenContractAddress,
      ],
      {
        initializer: 'initialize',
        txOverrides: {
          maxFeePerGas: await getMaxFeePerGas(ethOptions, hre),
          gasLimit: ethOptions.gasLimit,
          gasPrice: ethOptions.gasPrice,
        },
      }
    )) as unknown as AaveYield;

    await aaveYield.waitForDeployment();
    aaveYieldAddress = await aaveYield.getAddress();

    const implementationAddress = await hre.upgrades.erc1967.getImplementationAddress(aaveYieldAddress);
    console.log(`AaveYield proxy: ${aaveYieldAddress}, implementation: ${implementationAddress}`);

    const txHash = aaveYield.deploymentTransaction()?.hash;

    stateStore.setById('aaveYield-proxy', <DeployState>{ txHash, address: aaveYieldAddress });
    stateStore.setById('aaveYield-impl', <DeployState>{ address: implementationAddress });
    deploymentStore.setById('aaveYield', <DeploymentState>{
      address: aaveYieldAddress,
      implementation: implementationAddress,
    });
  }
}

async function deployEthYield(
  signer: Signer,
  ethConfig: EthYieldDeploymentConfig,
  ethOptions: EthOptions,
  hre: HardhatRuntimeEnvironment,
  stateStore: StateStore,
  deploymentStore: DeploymentStore
): Promise<void> {
  console.log(`Deploy EthYield`);

  const state = stateStore.getById('ethYield-proxy');
  if (state !== undefined) {
    console.log(`EthYield already deployed. Skip.`);
    return;
  }

  const ethYield = (await hre.upgrades.deployProxy(
    new EthYield__factory().connect(signer),
    [
      ethConfig.stETH,
      ethConfig.wETH9,
      ethConfig.eigenLayer.strategy,
      ethConfig.eigenLayer.strategyManager,
      ethConfig.eigenLayer.delegationManager,
      ethConfig.eigenLayer.operator,
    ],
    {
      initializer: 'initialize',
      txOverrides: {
        maxFeePerGas: await getMaxFeePerGas(ethOptions, hre),
        gasLimit: ethOptions.gasLimit,
        gasPrice: ethOptions.gasPrice,
      },
    }
  )) as unknown as EthYield;

  await ethYield.waitForDeployment();

  await ethYield
    .connect(signer)
    .initializeV2(
      ethConfig.lidoWithdrawalQueue,
      ethConfig.wardenHandler.axelarGateway,
      ethConfig.wardenHandler.axelarGasService,
      ethConfig.wardenHandler.evmChainName,
      ethConfig.wardenHandler.wardenChain,
      ethConfig.wardenHandler.wardenContractAddress,
      {
        maxFeePerGas: await getMaxFeePerGas(ethOptions, hre),
        gasLimit: ethOptions.gasLimit,
        gasPrice: ethOptions.gasPrice,
      }
    );

  const ethYieldAddress = await ethYield.getAddress();
  const implementationAddress = await hre.upgrades.erc1967.getImplementationAddress(ethYieldAddress);
  console.log(`EthYield proxy: ${ethYieldAddress}, implementation: ${implementationAddress}`);

  const txHash = ethYield.deploymentTransaction()?.hash;

  stateStore.setById('ethYield-proxy', <DeployState>{ txHash, address: ethYieldAddress });
  stateStore.setById('ethYield-impl', <DeployState>{ address: ethYieldAddress });
  deploymentStore.setById('ethYield', <DeploymentState>{
    address: ethYieldAddress,
    implementation: implementationAddress,
  });
}
