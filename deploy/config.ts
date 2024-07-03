import fs from 'fs';
import path from 'path';
import { isAddress, Provider } from 'ethers';
import {
  ERC20__factory,
  IDelegationManager__factory,
  IPool__factory,
  IStrategy__factory,
  IStrategyManager__factory,
} from '../typechain-types';

export interface Config {
  ethConnection: EthConnectionConfig;
  aaveYield?: AaveYieldConfig;
  ethYield?: EthYieldConfig;
}

export interface AaveYieldConfig {
  aavePool: string;
  tokens: TokenConfig[];
  enableWithdrawals: boolean;
}

export interface EthYieldConfig {
  stETH: string;
  wETH9: string;
  eigenLayer: EigenLayerConfig;
}

export interface EigenLayerConfig {
  strategy: string;
  strategyManager: string;
  delegationManager: string;
  operator: string;
}

export interface TokenConfig {
  address: string;
  symbol: string;
  decimals: number;
}

export interface EthConnectionConfig {
  ethOptions: EthOptions;
  chainId: number;
}

export interface EthOptions {
  gasLimit: number | null | undefined;
  gasPrice: number | null | undefined;
}

export const configAllowedKeys = new Set<string>(['ethConnection', 'aaveYield', 'ethYield']);

export async function loadDeployConfig(network: string, provider: Provider, dryRun: boolean): Promise<Config> {
  const configDir = path.join(__dirname, `data`, `configs`, network);

  if (!fs.existsSync(configDir)) {
    throw new Error(`Directory '${configDir}' does not exists`);
  }
  if (!fs.statSync(configDir).isDirectory()) {
    throw new Error(`Specified '${configDir}' is not a directory`);
  }
  const configFilename = path.join(configDir, 'config.json');
  if (!fs.existsSync(configFilename)) {
    throw new Error(`Deploy config is not exist! Filename: ${configFilename}`);
  }
  const config: Config = JSON.parse(fs.readFileSync(configFilename, 'utf-8'));

  await assertDeployConfigValidity(config, provider, dryRun);

  return config;
}

async function assertDeployConfigValidity(config: Config, provider: Provider, dryRun: boolean): Promise<void> {
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

  await assertAaveYieldDeployConfigValidity(config, provider);
  await assertEthYieldDeployConfigValidity(config, provider);
}

async function assertAaveYieldDeployConfigValidity(config: Config, provider: Provider): Promise<void> {
  const aave = config.aaveYield;
  if (aave === undefined) return;

  if (!isAddress(aave.aavePool)) {
    throw new Error(`Invalid Aave pool address! Address: "${aave.aavePool}"`);
  }
  const pool = IPool__factory.connect(aave.aavePool, provider);
  for (const token of aave.tokens) {
    await assertTokenConfig(token, provider);

    const reserveNormalizedIncome = await pool.getReserveNormalizedIncome(token.address);
    if (reserveNormalizedIncome === BigInt(0)) {
      throw new Error(`Token reserveNormalizedIncome == 0! Address: ${token.address}, symbol: ${token.symbol}`);
    }
  }
}

async function assertEthYieldDeployConfigValidity(config: Config, provider: Provider): Promise<void> {
  const ethYield = config.ethYield;
  if (ethYield === undefined) return;

  await assertTokenConfig(<TokenConfig>{ address: ethYield.wETH9, symbol: 'WETH', decimals: 18 }, provider);
  await assertTokenConfig(<TokenConfig>{ address: ethYield.stETH, symbol: 'stETH', decimals: 18 }, provider);

  const el = ethYield.eigenLayer;
  if (!isAddress(el.strategyManager)) {
    throw new Error(`Invalid EL StrategyManager address! Address: "${el.strategyManager}"`);
  }
  if (!isAddress(el.delegationManager)) {
    throw new Error(`Invalid EL DelegationManager address! Address: "${el.delegationManager}"`);
  }
  if (!isAddress(el.strategy)) {
    throw new Error(`Invalid EL Strategy address! Address: "${el.strategy}"`);
  }
  if (!isAddress(el.operator)) {
    throw new Error(`Invalid EL Operator address! Address: "${el.operator}"`);
  }

  const strategyManager = IStrategyManager__factory.connect(el.strategyManager, provider);
  const strategyIsWhitelistedForDeposit = await strategyManager.strategyIsWhitelistedForDeposit(el.strategy);
  if (!strategyIsWhitelistedForDeposit) {
    throw new Error(`EL Strategy is not whitelisted for deposits!`);
  }

  const strategy = IStrategy__factory.connect(el.strategy, provider);
  const strategyUnderlyingToken = await strategy.underlyingToken();
  if (strategyUnderlyingToken.toLowerCase() !== ethYield.stETH.toLowerCase()) {
    throw new Error(
      `EL Strategy underlying token != stETH.` +
        `Underlying token: ${strategyUnderlyingToken}, setETH: ${ethYield.stETH}`
    );
  }

  const delegationManager = IDelegationManager__factory.connect(el.delegationManager, provider);
  const isOperator = await delegationManager.isOperator(el.operator);
  if (!isOperator) {
    throw new Error(`EL operator invalid!.`);
  }
}

async function assertTokenConfig(token: TokenConfig, provider: Provider): Promise<void> {
  if (!isAddress(token.address)) {
    throw new Error(`Invalid token address! Address: "${token.address}", symbol: ${token.symbol}`);
  }
  const tokenContract = ERC20__factory.connect(token.address, provider);
  const symbol = await tokenContract.symbol();
  if (symbol !== token.symbol) {
    throw new Error(
      `Invalid token symbol! Address: ${token.address}, expected symbol: ${token.symbol}, actual: ${symbol}`
    );
  }
  const decimals = await tokenContract.decimals();
  if (Number(decimals) !== token.decimals) {
    throw new Error(
      `Invalid token decimals! Address: ${token.address}, expected decimals: ${token.decimals}, actual: ${decimals}`
    );
  }
}
