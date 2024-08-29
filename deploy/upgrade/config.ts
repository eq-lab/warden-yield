import fs from 'fs';
import path from 'path';
import { BaseContract, isAddress, Provider } from 'ethers';
import { assertWardenHandlerConfigValidity, EthConnectionConfig, WardenHandlerConfig } from '../config-common';
import { ILidoWithdrawalQueue__factory } from '../../typechain-types';

export interface UpgradeConfig {
  ethConnection: EthConnectionConfig;
  aaveYield?: AaveYieldUpgradeConfig;
  ethYield?: EthYieldUpgradeConfig;
}

export interface AaveYieldUpgradeConfig {
  underlyingToken: string;
  wardenHandler: WardenHandlerConfig;
}

export interface EthYieldUpgradeConfig {
  lidoWithdrawalQueue: string;
  wardenHandler: WardenHandlerConfig;
}

export const UPGRADE_CONFIG_FILENAME = 'upgrade-config.json';

export const configAllowedKeys = new Set<string>(['ethConnection', 'aaveYield', 'ethYield']);

export async function loadUpgradeConfig(network: string, provider: Provider, dryRun: boolean): Promise<UpgradeConfig> {
  const configDir = path.join(path.parse(__dirname).dir, `data`, `configs`, network);

  if (!fs.existsSync(configDir)) {
    throw new Error(`Directory '${configDir}' does not exist`);
  }
  if (!fs.statSync(configDir).isDirectory()) {
    throw new Error(`Specified '${configDir}' is not a directory`);
  }
  const configFilename = path.join(configDir, UPGRADE_CONFIG_FILENAME);
  if (!fs.existsSync(configFilename)) {
    throw new Error(`Upgrade config does not exist! Filename: ${configFilename}`);
  }
  const config: UpgradeConfig = JSON.parse(fs.readFileSync(configFilename, 'utf-8'));

  await assertUpgradeConfigValidity(config, provider, dryRun);

  return config;
}

async function assertUpgradeConfigValidity(config: UpgradeConfig, provider: Provider, dryRun: boolean): Promise<void> {
  const assertChainId = config.ethConnection.chainId;
  const network = await provider.getNetwork();

  if (!dryRun && Number(network.chainId) !== assertChainId) {
    throw new Error(`Chain id is invalid! Expected: ${assertChainId}, actual: ${network.chainId}`);
  }

  if (config.ethConnection.ethOptions.gasLimit === undefined) {
    config.ethConnection.ethOptions.gasLimit = null;
  }
  if (config.ethConnection.ethOptions.gasPrice === undefined) {
    config.ethConnection.ethOptions.gasPrice = null;
  }

  for (const key of Object.keys(config)) {
    if (!configAllowedKeys.has(key)) {
      throw new Error(`Unknown config field: "${key}". Allowed fields: [${configAllowedKeys}]`);
    }
  }

  await assertAaveYieldUpgradeConfigValidity(config, provider);
  await assertEthYieldUpgradeConfigValidity(config, provider);
}

async function assertAaveYieldUpgradeConfigValidity(config: UpgradeConfig, provider: Provider): Promise<void> {
  const aave = config.aaveYield;
  if (aave === undefined) return;

  if (!isAddress(aave.underlyingToken)) {
    throw new Error(`Invalid Aave underlyingToken: "${aave.underlyingToken}"`);
  }

  assertWardenHandlerConfigValidity(aave.wardenHandler, provider);
}

async function assertEthYieldUpgradeConfigValidity(config: UpgradeConfig, provider: Provider): Promise<void> {
  const ethYield = config.ethYield;
  if (ethYield === undefined) return;

  if (!isAddress(ethYield.lidoWithdrawalQueue)) {
    throw new Error(`Invalid Lido withdrawal queue: "${ethYield.lidoWithdrawalQueue}"`);
  }

  const lidoQueue = ILidoWithdrawalQueue__factory.connect(ethYield.lidoWithdrawalQueue, provider);
  await lidoQueue.MAX_STETH_WITHDRAWAL_AMOUNT(); // throws an error if address has no right method hash

  assertWardenHandlerConfigValidity(ethYield.wardenHandler, provider);
}
