import { EthOptions } from './config-common';
import { Provider } from 'ethers';

export async function getMaxFeePerGas(ethOptions: EthOptions, provider: Provider): Promise<number> {
  const blockNumber = await provider.provider.getBlockNumber();
  const lastBaseFeePerGas = (await provider.getBlock(blockNumber))?.baseFeePerGas;
  if (lastBaseFeePerGas == null) {
    throw new Error('Failed to fetch last block baseFeePerGas');
  }
  const multiplier = ethOptions.maxFeePerGasMultiplier ? ethOptions.maxFeePerGasMultiplier : 1;
  return Number(lastBaseFeePerGas) * multiplier;
}
