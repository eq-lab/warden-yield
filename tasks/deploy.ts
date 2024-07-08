import { task } from 'hardhat/config';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { loadDeployConfig } from '../deploy/config';
import { deployWardenYield } from '../deploy/src';

interface DeployArgs {
  networkName: string;
  creatorPrivateKey: string;
}

task('task:deploy', 'Deploy Yield proxies and implementations')
  .addParam<string>('networkName', 'Network name')
  .addParam<string>('creatorPrivateKey', 'Private key of contracts creator')
  .setAction(async (taskArgs: DeployArgs, hre: HardhatRuntimeEnvironment) => {
    const dryRun =
      hre.config.networks.hardhat.forking !== undefined ? hre.config.networks.hardhat.forking.enabled : false;

    const network = taskArgs.networkName.toLowerCase();
    if (!dryRun && hre.network.name.toLowerCase() !== network) {
      throw new Error(`The network from the config and from CLI "--network" must be same!`);
    }

    if (dryRun) {
      console.log(`Dry run command on fork`);
      const blockNumber = await hre.ethers.provider.getBlockNumber();
      console.log(`Fork block number: ${blockNumber}`);
    }

    const signer = new hre.ethers.Wallet(taskArgs.creatorPrivateKey, hre.ethers.provider);
    const config = await loadDeployConfig(network, signer.provider, dryRun);

    await deployWardenYield(signer, config, network, dryRun, hre);

    console.log(`Done!`);
  });
