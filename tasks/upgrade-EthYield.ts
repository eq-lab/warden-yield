import { task } from 'hardhat/config';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { loadDeployConfig } from '../deploy/config';
import { deployWardenYield, upgradeAaveYieldToV2, upgradeEthYield } from '../deploy/src';

interface UpgradeArgs {
  networkName: string;
  proxyAddress: string;
  newImplementationAddress: string;
  token: string;
  creatorPrivateKey: string;
}

task('task:upgrade-eth-yield', 'Upgrade EthYield from v1 to v2')
  .addParam<string>('networkName', 'Network name')
  .addParam<string>('creatorPrivateKey', 'Private key of contracts creator')
  // .addOptionalParam<number>('gasLimit', 'Gas limit')
  // .addOptionalParam<number>('gasPrice', 'Gas price')
  .setAction(async (taskArgs: UpgradeArgs, hre: HardhatRuntimeEnvironment) => {
    const dryRun =
      hre.config.networks.hardhat.forking !== undefined ? hre.config.networks.hardhat.forking.enabled : false;

    const network = taskArgs.networkName.toLowerCase();
    if (!dryRun && hre.network.name.toLowerCase() !== network) {
      throw new Error(`The network from the config and from CLI "--network" must be same!`)
    }
    if (taskArgs.token !== 'usdc' && taskArgs.token !== 'usdt') {
      throw new Error(`Token must be set as 'usdt' or 'usdc'`);
    }

    if (dryRun) {
      console.log(`Dry run command on fork`);
      const blockNumber = await hre.ethers.provider.getBlockNumber();
      console.log(`Fork block number: ${blockNumber}`);
    }

    const config = await loadDeployConfig(network, signer.provider, dryRun);
    if (config.ethYield?.lidoWithdrawalQueue === undefined) {
      throw new Error('Unknown lidoWithdrawalQueue address');
    }

    const signer = new hre.ethers.Wallet(taskArgs.creatorPrivateKey, hre.ethers.provider);
    const balanceBefore = await signer.provider.getBalance(signer.address);
    console.log(`Balance before: ${hre.ethers.formatEther(balanceBefore)} Eth\n`);
    
    await upgradeEthYield(
      signer,
      config.ethYield,
      config.ethConnection,
      hre,
      network,
      dryRun,
    );

    const balanceAfter = await signer.provider.getBalance(signer.address);
    console.log(`\nBalance after: ${hre.ethers.formatEther(balanceAfter)} Eth`);

    console.log(`Spent: ${hre.ethers.formatEther(balanceBefore - balanceAfter)} Eth`);
    console.log(`Done!`);
  });
