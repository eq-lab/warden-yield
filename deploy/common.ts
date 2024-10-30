import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { EthOptions } from './config-common';

export async function getMaxFeePerGas(ethOptions: EthOptions, hre: HardhatRuntimeEnvironment): Promise<number> {
  const blockNumber = await hre.ethers.provider.provider.getBlockNumber();
  const lastBaseFeePerGas = (await hre.ethers.provider.getBlock(blockNumber))?.baseFeePerGas;
  if (lastBaseFeePerGas == null) {
    throw new Error('Failed to fetch last block baseFeePerGas');
  }
  const multiplier = ethOptions.maxFeePerGasMultiplier ? ethOptions.maxFeePerGasMultiplier : 1;
  return Number(lastBaseFeePerGas) * multiplier;
}
