import { task } from 'hardhat/config';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { loadDeployConfig } from '../deploy/config';
import { deployAaveYieldImplementationV2, deployWardenYield, upgradeAaveYieldToV2 } from '../deploy/src';
import { AaveYieldV2__factory, ERC20__factory } from '../typechain-types';

interface TestCase {
  from: string;
  proxy: string;
  tokenName: string;
  tokenAddress: string;
}

const usdcTestCase: TestCase = {
  from: '0xDc287d4CE3758B99cB4b4460D07C375E9cf50AB2',
  proxy: '0x0259044395FE54d8aFe28354Ac737EB216064cF9',
  tokenName: 'usdc',
  tokenAddress: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48'
}
const usdtTestCase: TestCase = {
  from: '',
  proxy: '',
  tokenName: 'usdt',
  tokenAddress: ''
}

interface UpgradeArgs {
  networkName: string;
  testCase: string;
  creatorPrivateKey: string;
}

task('task:test-upgrade-AaveYield-to-v2', 'Upgrade AaeYield from v1 to v2')
  .addParam<string>('networkName', 'Network name')
  .addParam<string>('testCase', 'usdc or usdt')
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
    let testCase;
    if (taskArgs.testCase === 'usdc') {
      testCase = usdcTestCase;
    } else if (taskArgs.testCase === 'usdt') {
      testCase = usdtTestCase;
    } else {
      throw new Error(`Token must be set as 'usdt' or 'usdc'`);
    }
    
    if (!dryRun) {
        throw new Error(`This task can be run only on fork`);
    }
    console.log(`Dry run command on fork`);
    const blockNumber = await hre.ethers.provider.getBlockNumber();
    console.log(`Fork block number: ${blockNumber}`);
    

    const signer = new hre.ethers.Wallet(taskArgs.creatorPrivateKey, hre.ethers.provider);
    const balanceBefore = await signer.provider.getBalance(signer.address);
    console.log(`Balance before: ${hre.ethers.formatEther(balanceBefore)} Eth\n`);
    
    const v2implAddress = await deployAaveYieldImplementationV2(signer, network, dryRun, hre);

    await upgradeAaveYieldToV2(
      signer,
      testCase.proxy,
      v2implAddress,
      testCase.tokenName,
      network,
      dryRun,
      hre);
    
    await simulateWithdrawal(usdcTestCase, hre);

    const balanceAfter = await signer.provider.getBalance(signer.address);
    console.log(`\nBalance after: ${hre.ethers.formatEther(balanceAfter)} Eth`);

    console.log(`Spent: ${hre.ethers.formatEther(balanceBefore - balanceAfter)} Eth`);
    console.log(`Done!`);
  });

  async function simulateWithdrawal(testCase: TestCase, hre: HardhatRuntimeEnvironment): Promise<void> {
    const { ethers, network } = hre;
    await network.provider.request({
      method: "hardhat_impersonateAccount",
      params: [testCase.from],
    });
    const signer = await ethers.getSigner(testCase.from);

    const aaveYield = AaveYieldV2__factory.connect(testCase.proxy, signer);
    const stakedAmountBefore = await aaveYield.userStakedAmount(testCase.from, testCase.tokenAddress);

    const usdc = ERC20__factory.connect(testCase.tokenAddress, signer);
    const userBalanceBefore = await usdc.balanceOf(testCase.from);
    const tx = await aaveYield.withdraw(testCase.tokenAddress);
    await tx.wait();
    const userBalanceAfter = await usdc.balanceOf(testCase.from);

    const userBalanceDelta = userBalanceAfter - userBalanceBefore;
    console.log(`User balance delta: ${ethers.formatUnits(userBalanceDelta, 6)} USDC`)
    
    const stakedAmountAfter = await aaveYield.userStakedAmount(testCase.from, testCase.tokenAddress);
    console.log(`User staked amount after: ${ethers.formatUnits(stakedAmountAfter, 6)} USDC`)

    if (stakedAmountBefore !== userBalanceDelta) {
      throw Error(`stakedAmountBefore !== userBalanceDelta`)
    }
    //
  }