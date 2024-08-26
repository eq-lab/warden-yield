import { task } from 'hardhat/config';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { upgradeWardenYield, loadUpgradeConfig } from '../deploy/src';

interface UpgradeArgs {
  networkName: string;
  creatorPrivateKey: string;
}

task('task:upgrade', 'Deploy Yield proxies and implementations')
  .addParam<string>('networkName', 'Network name')
  .addParam<string>('creatorPrivateKey', 'Private key of contracts creator')
  .setAction(async (taskArgs: UpgradeArgs, hre: HardhatRuntimeEnvironment) => {
    const dryRun =
      hre.config.networks.hardhat.forking !== undefined ? hre.config.networks.hardhat.forking.enabled : false;

    const network = taskArgs.networkName.toLowerCase();
    if (!dryRun && hre.network.name.toLowerCase() !== network) {
      throw new Error(`The network from the config and from CLI "--network" must be same!`);
    }

    const provider = hre.ethers.provider;

    if (dryRun) {
      console.log(`Dry run command on fork`);
      const blockNumber = await provider.getBlockNumber();
      console.log(`Fork block number: ${blockNumber}`);
    }

    const signer = new hre.ethers.Wallet(taskArgs.creatorPrivateKey, provider);
    const config = await loadUpgradeConfig(network, provider, dryRun);

    await upgradeWardenYield(signer, config, network, dryRun, hre);

    console.log(`Done!`);
  });
