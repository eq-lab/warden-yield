import { Contract, Signer } from 'ethers';
import { AaveYieldConfig, Config, EthConnectionConfig, EthYieldConfig } from './config';
import { AaveYield, AaveYield__factory, AaveYieldV2__factory, EthYield, EthYield__factory, UUPSUpgradeable__factory } from '../typechain-types';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { createDefaultBaseState, DeployState, StateFile, StateStore } from './state-store';
import { SimpleLogger } from './logger';
import fs from 'fs';
import path from 'path';
import { createDefaultBaseDeployment, DeploymentFile, DeploymentState, DeploymentStore } from './deployment-store';

export async function deployWardenYield(
  signer: Signer,
  config: Config,
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

  if (config.aaveYield !== undefined) {
    await deployAaveYield(signer, config.aaveYield, config.ethConnection, hre, stateStore, deploymentStore);
  }
  if (config.ethYield !== undefined) {
    await deployEthYield(signer, config.ethYield, config.ethConnection, hre, stateStore, deploymentStore);
  }

  console.log(`State file: \n${stateStore.stringify()}`);
  console.log(`Deployment file: \n${deploymentStore.stringify()}`);
}

async function deployAaveYield(
  signer: Signer,
  aaveConfig: AaveYieldConfig,
  ethConnectionConfig: EthConnectionConfig,
  hre: HardhatRuntimeEnvironment,
  stateStore: StateStore,
  deploymentStore: DeploymentStore
): Promise<void> {
  const allowedTokens = aaveConfig.tokens.map((x) => x.address);

  console.log(`Deploy AaveYield. AavePool: ${aaveConfig.aavePool}, allowedTokens: [${allowedTokens}]`);

  const blockNumber = await hre.ethers.provider.provider.getBlockNumber();

  const state = stateStore.getById('aaveYield-proxy');
  let aaveYieldAddress: string;
  if (state !== undefined) {
    console.log(`AaveYield already deployed. Skip.`);
    aaveYieldAddress = state.address;
  } else {
    const aaveYield = (await hre.upgrades.deployProxy(
      await new AaveYield__factory().connect(signer),
      [aaveConfig.aavePool, allowedTokens],
      {
        initializer: 'initialize',
        txOverrides: {
          gasLimit: ethConnectionConfig.ethOptions.gasLimit,
          gasPrice: ethConnectionConfig.ethOptions.gasPrice,
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

  if (aaveConfig.enableWithdrawals) {
    const aaveYield = (await AaveYield__factory.connect(aaveYieldAddress, signer)) as AaveYield;
    const areWithdrawalsEnabled = await aaveYield.connect(signer.provider).areWithdrawalsEnabled();
    if (!areWithdrawalsEnabled) {
      console.log(`Enabling withdrawals...`);
      await aaveYield.enableWithdrawals({
        gasLimit: ethConnectionConfig.ethOptions.gasLimit,
        gasPrice: ethConnectionConfig.ethOptions.gasPrice,
      });
    }
  }
}

export async function deployAaveYieldImplementationV2(
  signer: Signer,
  network: string,
  dryRun: boolean,
  hre: HardhatRuntimeEnvironment
): Promise<string> {
  const statesDirName = 'states';
  const stateFileName = getStateFileName(network, statesDirName);
  const actualStateFile = path.join(__dirname, `data`, `configs`, network, stateFileName);
  const actualDeploymentFile = path.join(__dirname, `data`, `contracts`, `${network}.json`);

  const logger = new SimpleLogger((x) => console.error(x));
  const stateStore = new StateFile(
    'AaveYieldV2',
    createDefaultBaseState,
    actualStateFile,
    !dryRun,
    logger
  ).createStateStore();

  const deploymentStore = new DeploymentFile(
    'AaveYieldV2',
    createDefaultBaseDeployment,
    actualDeploymentFile,
    !dryRun,
    logger
  ).createDeploymentStore();

  const address = await deployAaveYieldImplementationV2Impl(signer, hre, stateStore, deploymentStore);

  console.log(`State file: \n${stateStore.stringify()}`);
  console.log(`Deployment file: \n${deploymentStore.stringify()}`);

  return address;
}

async function deployAaveYieldImplementationV2Impl(
  signer: Signer,
  hre: HardhatRuntimeEnvironment,
  stateStore: StateStore,
  deploymentStore: DeploymentStore
): Promise<string> {
  console.log(`Deploy AaveYield V2 implementation`);

  const blockNumber = await hre.ethers.provider.provider.getBlockNumber();
  console.log(`Block number: ${blockNumber}`);
  const implV2 = await new AaveYieldV2__factory().connect(signer).deploy();
  await implV2.waitForDeployment();
  const implV2Address = await implV2.getAddress();
  console.log(`AaveYield V2 implementation: ${implV2Address}`);

  const txHash = implV2.deploymentTransaction()?.hash;
  stateStore.setById('aaveYield-v2-impl', <DeployState>{ txHash, address: implV2Address });
  deploymentStore.setById('aaveYield-v2-impl', <DeploymentState>{
    address: implV2Address,
  });
  return implV2Address;
}

export async function upgradeAaveYieldToV2(
  signer: Signer,
  proxyAddress: string,
  newImplementationAddress: string,
  token: string,
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

  await upgradeAaveYieldToV2Impl(signer, proxyAddress, newImplementationAddress, token, hre, stateStore, deploymentStore);

  console.log(`State file: \n${stateStore.stringify()}`);
  console.log(`Deployment file: \n${deploymentStore.stringify()}`);
}

async function upgradeAaveYieldToV2Impl(
  signer: Signer,
  proxyAddress: string,
  newImplementationAddress: string,
  token: string,
  hre: HardhatRuntimeEnvironment,
  stateStore: StateStore,
  deploymentStore: DeploymentStore
): Promise<void> {
  console.log(`Deploy AaveYield V2`);

  const blockNumber = await hre.ethers.provider.provider.getBlockNumber();
  console.log(`Block number: ${blockNumber}`);

  const proxy = UUPSUpgradeable__factory.connect(proxyAddress, signer);
  console.log(`Upgrade proxy...`);
  const tx = await proxy.upgradeToAndCall(newImplementationAddress, "0x");
  console.log(`Transaction hash: ${tx.hash}`);
  await tx.wait();
  
  console.log(`AaveYield proxy: ${proxyAddress}, new implementation: ${newImplementationAddress}`);
  stateStore.setById(`aaveYield-impl-v2`, <DeployState>{ txHash: tx.hash, address: newImplementationAddress });
  deploymentStore.setById(`aaveYield-${token}`, <DeploymentState>{
    address: proxyAddress,
    implementation: newImplementationAddress,
  });
}

async function deployEthYield(
  signer: Signer,
  ethConfig: EthYieldConfig,
  ethConnectionConfig: EthConnectionConfig,
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

  const blockNumber = await hre.ethers.provider.getBlockNumber();

  const ethYield = (await hre.upgrades.deployProxy(
    await new EthYield__factory().connect(signer),
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
        gasLimit: ethConnectionConfig.ethOptions.gasLimit,
        gasPrice: ethConnectionConfig.ethOptions.gasPrice,
      },
    }
  )) as unknown as EthYield;

  await ethYield.waitForDeployment();

  const ethYieldAddress = await ethYield.getAddress();
  const implementationAddress = await hre.upgrades.erc1967.getImplementationAddress(ethYieldAddress);
  console.log(`EthYield proxy: ${ethYieldAddress}, implementation: ${implementationAddress}`);

  const txHash = ethYield.deploymentTransaction()?.hash;

  stateStore.setById('ethYield-proxy', <DeployState>{ txHash, address: ethYieldAddress });
  stateStore.setById('ethYield-impl', <DeployState>{ address: implementationAddress });
  deploymentStore.setById('ethYield', <DeploymentState>{
    address: ethYieldAddress,
    implementation: implementationAddress,
  });
}

export async function upgradeEthYield(
  signer: Signer,
  ethConfig: EthYieldConfig,
  ethConnectionConfig: EthConnectionConfig,
  hre: HardhatRuntimeEnvironment,
  network: string,
  dryRun: boolean,
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

  const state = stateStore.getById('ethYield-proxy');
  if (state === undefined) {
    throw new Error('Failed to obtain EthYield proxy address');
  }

  const ethYieldAddress = state.address;
  const implementationAddressBefore = await hre.upgrades.erc1967.getImplementationAddress(ethYieldAddress);
  const contract = await hre.upgrades.upgradeProxy(ethYieldAddress, new EthYield__factory().connect(signer), {
    call: {
      fn: 'initializeV2',
      args: [ethConfig.lidoWithdrawalQueue],
    },
    txOverrides: {
      gasLimit: ethConnectionConfig.ethOptions.gasLimit,
      gasPrice: ethConnectionConfig.ethOptions.gasPrice,
    },
  });
  await contract.waitForDeployment();
  const implementationAddress = await hre.upgrades.erc1967.getImplementationAddress(ethYieldAddress);
  console.log(`EthYield proxy: ${ethYieldAddress}, old implementation: ${implementationAddressBefore}, new implementation: ${implementationAddress}`);

  stateStore.setById('ethYield-impl', <DeployState>{ address: implementationAddress });
  deploymentStore.setById('ethYield', <DeploymentState>{
    address: ethYieldAddress,
    implementation: implementationAddress,
  });
}

function getStateFileName(network: string, statesDirName: string): string {
  const dirName = path.join(__dirname, `data`, `configs`, network, statesDirName);
  if (!fs.existsSync(dirName)) {
    fs.mkdirSync(dirName);
  }

  if (!fs.statSync(dirName).isDirectory()) {
    throw new Error(`Not a directory: ${dirName}`);
  }

  let stateFilename = getLatestStateFileName(dirName);

  if (stateFilename === undefined) {
    const fileName = path.join(statesDirName, generateStateFileName(dirName));
    console.log(`Using new generated state file '${fileName}'`);
    return fileName;
  } else {
    const fileName = path.join(statesDirName, stateFilename);
    console.log(`Using latest state file '${fileName}'`);
    return fileName;
  }
}

function getLatestStateFileName(dirName: string): string | undefined {
  const fileNames = fs.readdirSync(dirName);
  const files = fileNames
    .map((x) => ({
      name: x,
      extension: path.extname(x),
      mtimeNs: fs.statSync(path.join(dirName, x), { bigint: true }).mtimeNs,
    }))
    .filter((x) => x.extension === '.json')
    .sort((a, b) => Number(b.mtimeNs - a.mtimeNs));

  if (files.length === 0) {
    return undefined;
  }

  return files[0].name;
}

function generateStateFileName(dirName: string): string {
  const now = new Date();
  const year = now.getUTCFullYear();
  const month = 1 + now.getUTCMonth();
  const day = now.getUTCDate();
  const dateStr = `${year}-${month.toString().padStart(2, '0')}-${day.toString().padStart(2, '0')}`;

  let fileName = `${dateStr}.json`;
  if (fs.existsSync(path.join(dirName, fileName))) {
    const maxCount = 99;
    let n = 1;
    while (fs.existsSync(path.join(dirName, fileName))) {
      if (n === maxCount) {
        throw new Error('Too much state files today');
      }
      fileName = `${dateStr}_${n.toString().padStart(2, '0')}.json`;
      n++;
    }
  }
  return fileName;
}
