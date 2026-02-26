import { task } from 'hardhat/config';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { deployAaveYieldImplementationV2 } from '../deploy/src';

interface DeployArgs {
  networkName: string;
  creatorPrivateKey: string;
}

task('task:deploy-AaveYield-v2-impl', 'Deploy AaveYield v2 implementation')
  .addParam<string>('networkName', 'Network name')
  .addParam<string>('creatorPrivateKey', 'Private key of contracts creator')
  .setAction(async (taskArgs: DeployArgs, hre: HardhatRuntimeEnvironment) => {
    const dryRun =
      hre.config.networks.hardhat.forking !== undefined ? hre.config.networks.hardhat.forking.enabled : false;

    const network = taskArgs.networkName.toLowerCase();
    if (!dryRun && hre.network.name.toLowerCase() !== network) {
      throw new Error(`The network from the config and from CLI "--network" must be same!`)
    }

    if (dryRun) {
      console.log(`Dry run command on fork`);
      const blockNumber = await hre.ethers.provider.getBlockNumber();
      console.log(`Fork block number: ${blockNumber}`);
    }

    const signer = new hre.ethers.Wallet(taskArgs.creatorPrivateKey, hre.ethers.provider);
    const balanceBefore = await signer.provider.getBalance(signer.address);
    console.log(`Balance before: ${hre.ethers.formatEther(balanceBefore)} Eth`);

    await deployAaveYieldImplementationV2(signer, network, dryRun, hre);

    const balanceAfter = await signer.provider.getBalance(signer.address);
    console.log(`Balance after: ${hre.ethers.formatEther(balanceAfter)} Eth`);

    console.log(`Spent: ${hre.ethers.formatEther(balanceBefore - balanceAfter)} Eth`);
    console.log(`Done!`);
  });
