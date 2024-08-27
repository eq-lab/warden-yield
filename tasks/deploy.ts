import { task } from 'hardhat/config';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { deployWardenYield, loadDeployConfig } from '../deploy/src';
import { Signer } from 'ethers';

interface DeployArgs {
  networkName: string;
  creatorKey: string;
  isPrivateKey: boolean;
}

task('task:deploy', 'Deploy Yield proxies and implementations')
  .addParam<string>('networkName', 'Network name')
  .addParam<string>('creatorKey', 'Private or public key of contracts creator')
  .addFlag('isPrivateKey', 'If passed `creatorKey` is the private one')
  .setAction(async (taskArgs: DeployArgs, hre: HardhatRuntimeEnvironment) => {
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

    let signer: Signer;
    if (taskArgs.isPrivateKey) {
      signer = new hre.ethers.Wallet(taskArgs.creatorKey, provider);
      console.log(`Signer from private key: ${await signer.getAddress()}`)
    } else if (dryRun) {
      console.log(`Impersonating signer: ${taskArgs.creatorKey}`);
      signer = await hre.ethers.getImpersonatedSigner(taskArgs.creatorKey);
    } else {
      throw new Error("Can't impersonate signer while not dry-running");
    }
  
    const config = await loadDeployConfig(network, provider, dryRun);

    await deployWardenYield(signer, config, network, dryRun, hre);

    console.log(`Done!`);
  });
