import { task } from 'hardhat/config';
import { loadDeployConfig } from '../deploy/config';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { deployWardenYield } from '../deploy/src';

interface DeployArgs {
  deployDir: string;
  creator: string;
}

task('task:deploy', 'Deploy Yield proxies and implementations')
  .addParam<string>('deployDir', 'Deploy dir with config')
  .addParam<string>('creatorPrivateKey', 'Private key of contracts creator')
  .setAction(async (taskArgs: DeployArgs, hre: HardhatRuntimeEnvironment) => {
    const dryRun =
      hre.config.networks.hardhat.forking !== undefined ? hre.config.networks.hardhat.forking.enabled : false;

    if (dryRun) {
      console.log(`Dry run command on fork`);
      const blockNumber = await hre.ethers.provider.getBlockNumber();
      console.log(`Fork block number: ${blockNumber}`);
    }

    const signer = new hre.ethers.Wallet(taskArgs.creator, hre.ethers.provider);
    const config = await loadDeployConfig(taskArgs.deployDir, signer.provider, dryRun);

    await deployWardenYield(signer, config, taskArgs.deployDir, dryRun, hre);

    console.log(`Done!`);
  });
