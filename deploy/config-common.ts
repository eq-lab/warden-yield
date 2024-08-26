import { isAddress, Provider } from 'ethers';
import { ERC20__factory } from '../typechain-types';

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

export async function assertTokenConfig(token: TokenConfig, provider: Provider): Promise<void> {
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
