import { task } from 'hardhat/config';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { loadDeployConfig } from '../deploy/config';
import { deployAaveYieldImplementationV2, deployWardenYield, upgradeAaveYieldToV2 } from '../deploy/src';
import { AaveYieldV2, AaveYieldV2__factory, ERC20, ERC20__factory } from '../typechain-types';

interface TestCase {
  users: string[];
  proxy: string;
  tokenName: string;
  tokenAddress: string;
  lptAddress: string;
}

const usdcTestCase: TestCase = {
  users: [
    '0x40b88b09610487A26b18FB52DBe319D1268fCa22',
    '0xbbC02dbD39941FFFa605171A977D44C6BAF915c2', 
    '0xDc287d4CE3758B99cB4b4460D07C375E9cf50AB2'
  ],
  proxy: '0x0259044395FE54d8aFe28354Ac737EB216064cF9',
  tokenName: 'usdc',
  tokenAddress: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  lptAddress: '0x98C23E9d8f34FEFb1B7BD6a91B7FF122F4e16F5c'
}
const usdtTestCase: TestCase = {
  users: ['0x40b88b09610487A26b18FB52DBe319D1268fCa22'],
  proxy: '0x0F9d2C03AD21a30746A4b4f07919e1C5F3641F35',
  tokenName: 'usdt',
  tokenAddress: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
  lptAddress: '0x23878914EFE38d27C4D67Ab83ed1b93A74D4086a'
}

interface YieldStats {
  totalUnderlyingAmount: bigint;
  totalLptAmount: bigint;
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
    
    await simulateWithdrawal(testCase, hre);

    const balanceAfter = await signer.provider.getBalance(signer.address);
    console.log(`\nBalance after: ${hre.ethers.formatEther(balanceAfter)} Eth`);

    console.log(`Spent: ${hre.ethers.formatEther(balanceBefore - balanceAfter)} Eth`);
    console.log(`Done!`);
  });

  async function simulateWithdrawal(testCase: TestCase, hre: HardhatRuntimeEnvironment): Promise<void> {
    const { ethers, network } = hre;

    for (const user of testCase.users){
      console.log(`\n\nUSER: ${user}`);
      await network.provider.request({
        method: "hardhat_impersonateAccount",
        params: [user],
      });
      
      const signer = await ethers.getSigner(user);
      const aaveYield = AaveYieldV2__factory.connect(testCase.proxy, signer);
      const lpt = ERC20__factory.connect(testCase.lptAddress, signer);

      const statsBefore = await collectYieldStats(aaveYield, lpt, testCase);
      console.log(`Stats before tx:`);
      printYieldStats(statsBefore, testCase, hre);

      const sharesAmountBefore = await aaveYield.userShares(user, testCase.tokenAddress);
      console.log(`User shares amount: ${ethers.formatUnits(sharesAmountBefore, 6)}`)

      const stakedAmountBefore = await aaveYield.userStakedAmount(user, testCase.tokenAddress);
      console.log(`User staked amount: ${ethers.formatUnits(stakedAmountBefore, 6)} ${testCase.tokenName}`)

      const canBeWithdrawBefore = await aaveYield.getUserUnderlyingAmount(user, testCase.tokenAddress);
      console.log(`User can withdraw: ${ethers.formatUnits(canBeWithdrawBefore, 6)} ${testCase.tokenName}`)

      const usdc = ERC20__factory.connect(testCase.tokenAddress, signer);
      const userBalanceBefore = await usdc.balanceOf(user);
      const tx = await aaveYield.withdraw(testCase.tokenAddress);
      await tx.wait();
      const userBalanceAfter = await usdc.balanceOf(user);

      const userBalanceDelta = userBalanceAfter - userBalanceBefore;
      console.log(`User balance delta: ${ethers.formatUnits(userBalanceDelta, 6)} ${testCase.tokenName}`)
      
      const stakedAmountAfter = await aaveYield.userStakedAmount(user, testCase.tokenAddress);
      console.log(`User staked amount after: ${ethers.formatUnits(stakedAmountAfter, 6)} ${testCase.tokenName}`)

      const statsAfter = await collectYieldStats(aaveYield, lpt, testCase);
      console.log(`Stats after tx:`);
      printYieldStats(statsAfter, testCase, hre);
      console.log(`Stats delta:`);
      printYieldStatsDelta(statsBefore, statsAfter, testCase, hre);
      

      // if (stakedAmountBefore !== userBalanceDelta) {
      //   throw Error(`stakedAmountBefore !== userBalanceDelta`)
      // }
    }

    const signer = await ethers.getSigner(testCase.users[0]);
    const aaveYield = AaveYieldV2__factory.connect(testCase.proxy, signer);
    const lpt = ERC20__factory.connect(testCase.lptAddress, signer);
    const finalStats = await collectYieldStats(aaveYield, lpt, testCase);
    console.log(`\n! FINAL STATS:`)
    printYieldStats(finalStats, testCase, hre);
    //
  }

  async function collectYieldStats(aaveYield: AaveYieldV2, lpt: ERC20, testCase: TestCase): Promise<YieldStats> {
    const totalUnderlyingAmount = await aaveYield.getTotalUnderlyingAmount(testCase.tokenAddress);
    const totalLptAmount = await lpt.balanceOf(testCase.proxy);
    return {totalUnderlyingAmount, totalLptAmount};
  }

  function printYieldStats(stats: YieldStats, testCase: TestCase, hre: HardhatRuntimeEnvironment): void {
    const { ethers } = hre;
    console.log(` - Total underlying amount: ${ethers.formatUnits(stats.totalUnderlyingAmount, 6)} ${testCase.tokenName}`)
    console.log(` - Total LPT amount: ${ethers.formatUnits(stats.totalLptAmount, 6)} LPT`)
  }

  function printYieldStatsDelta(oldStats: YieldStats, newStats: YieldStats, testCase: TestCase, hre: HardhatRuntimeEnvironment): void {
    const { ethers } = hre;
    const totalUnderlyingAmountDelta = oldStats.totalUnderlyingAmount - newStats.totalUnderlyingAmount;
    console.log(` - Total underlying amount DELTA: ${ethers.formatUnits(totalUnderlyingAmountDelta, 6)} ${testCase.tokenName}`)
    const totalLptAmountDelta = oldStats.totalLptAmount - newStats.totalLptAmount;
    console.log(` - Total LPT amount DELTA: ${ethers.formatUnits(totalLptAmountDelta, 6)} LPT`)
  }