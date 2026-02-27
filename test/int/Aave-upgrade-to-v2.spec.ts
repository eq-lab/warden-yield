import { expect } from 'chai';
import * as helpers from '@nomicfoundation/hardhat-network-helpers';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers, network, upgrades } from 'hardhat';
import { parseUnits } from 'ethers';
import { createAaveEthFork, deployAaveYieldContract } from '../shared/fixtures';
import { EthAddressData, setTokenBalance, USER_WARDEN_ADDRESS } from '../shared/utils';
import { HardhatEthersSigner } from '@nomicfoundation/hardhat-ethers/signers';
import { AaveYield, AaveYield__factory, AaveYieldUpgradeTest__factory, AaveYieldV2, AaveYieldV2__factory, ERC20, ERC20__factory, IAToken, IAToken__factory } from '../../typechain-types';

const MAX_DELTA = 30n;
const deployerAddress = '0xD36c2b552cF65c74F14CFF71fc9aD08D755D3947';

const aaveYieldUsdcAddress = '0x0259044395FE54d8aFe28354Ac737EB216064cF9';
const usdcAddress = '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48';
const aUsdcAddress = '0x98C23E9d8f34FEFb1B7BD6a91B7FF122F4e16F5c';
const aaveYieldUsdcUsers = [
  '0x40b88b09610487A26b18FB52DBe319D1268fCa22',
  '0xbbC02dbD39941FFFa605171A977D44C6BAF915c2', 
  '0xDc287d4CE3758B99cB4b4460D07C375E9cf50AB2'
];

const aaveYieldUsdtAddress = '0x0F9d2C03AD21a30746A4b4f07919e1C5F3641F35';
const usdtAddress = '0xdAC17F958D2ee523a2206206994597C13D831ec7';
const aUsdtAddress = '0x23878914EFE38d27C4D67Ab83ed1b93A74D4086a';
const aaveYieldUsdtUsers = [
  '0x40b88b09610487A26b18FB52DBe319D1268fCa22',
  '0xbbC02dbD39941FFFa605171A977D44C6BAF915c2'
];

async function createYieldStorageAssert(aaveYield: AaveYieldV2, aToken: IAToken, account: string, token: string) {
  const stakedAmountBefore = await aaveYield.userStakedAmount(account, token);
  const totalStakedAmountBefore = await aaveYield.totalStakedAmount(token);
  const sharesBefore = await aaveYield.userShares(account, token);
  const totalSharesBefore = await aaveYield.totalShares(token);
  const aTokenScaledBalanceBefore = await aToken.scaledBalanceOf(await aaveYield.getAddress());

  return async (stakedDeltaExpected: bigint) => {
    const stakedAmount = await aaveYield.userStakedAmount(account, token);
    const totalStakedAmount = await aaveYield.totalStakedAmount(token);
    expect(stakedAmount).to.be.closeTo(stakedAmountBefore + stakedDeltaExpected, MAX_DELTA);
    expect(totalStakedAmount).to.be.closeTo(totalStakedAmountBefore + stakedDeltaExpected, MAX_DELTA);

    const shares = await aaveYield.userShares(account, token);
    const totalShares = await aaveYield.totalShares(token);

    const aTokenScaledBalanceDelta =
      (await aToken.scaledBalanceOf(await aaveYield.getAddress())) - aTokenScaledBalanceBefore;

    expect(shares).to.be.closeTo(sharesBefore + aTokenScaledBalanceDelta, MAX_DELTA);
    expect(totalShares).to.be.closeTo(totalSharesBefore + aTokenScaledBalanceDelta, MAX_DELTA);
  };
}

async function withdraw(
  aaveYieldV2: AaveYieldV2,
  signer: HardhatEthersSigner,
  aToken: IAToken,
  token: ERC20
): Promise<void> {
  const aaveYieldAddress = await aaveYieldV2.getAddress();
  const tokenAddress = await token.getAddress();

  // state before withdraw
  const aEthScaledBalanceBefore = await aToken.scaledBalanceOf(aaveYieldAddress);
  const assertYieldStorage = await createYieldStorageAssert(aaveYieldV2, aToken, signer.address, tokenAddress);
  const availableToWithdraw = await aaveYieldV2.getUserUnderlyingAmount(signer.address, tokenAddress);
  const staked = await aaveYieldV2.userStakedAmount(signer.address, tokenAddress);
  const scaledBalance = await aaveYieldV2.userShares(signer.address, tokenAddress);

  console.log(` - staked balance: ${ethers.formatUnits(staked, 6)}`)
  const tokenBalanceBefore = await token.balanceOf(signer.address)

  await aaveYieldV2.connect(signer).withdraw(token);

  const tokenBalanceAfter = await token.balanceOf(signer.address)
  const aTokenBalanceAfter = await aToken.scaledBalanceOf(aaveYieldAddress);
  expect(aTokenBalanceAfter).to.be.closeTo(aEthScaledBalanceBefore - scaledBalance, MAX_DELTA);

  const tokenBalanceDelta = tokenBalanceAfter - tokenBalanceBefore;
  console.log(` - token balance delta: ${ethers.formatUnits(tokenBalanceDelta, 6)}`)
  expect(tokenBalanceDelta).to.be.closeTo(availableToWithdraw, MAX_DELTA);
  
  await assertYieldStorage(-staked);
}

async function initBalance(account: string, token: ERC20, balanceStr: string): Promise<bigint> {
  const decimals = await token.decimals();
  const balance = parseUnits(balanceStr, decimals);
  await setTokenBalance(await token.getAddress(), account, balance);
  expect(await token.balanceOf(account)).to.be.eq(balance);
  return balance;
}


describe('AaveYieldV2 actions', () => {
  let deployer;
  let aaveYieldUsdcV2: AaveYieldV2;
  let aaveYieldUsdtV2: AaveYieldV2;
  let usdc: ERC20;
  let aUsdc: IAToken;
  let usdt: ERC20;
  let aUsdt: IAToken;
  
  
  before(async () => {
    await network.provider.request({
      method: "hardhat_impersonateAccount",
      params: [deployerAddress],
    });
    deployer = await ethers.getSigner(deployerAddress);
    
    aaveYieldUsdcV2 = await upgrades.upgradeProxy(
      aaveYieldUsdcAddress,
      new AaveYieldV2__factory().connect(deployer),
    ) as unknown as AaveYieldV2;

    aaveYieldUsdtV2 = await upgrades.upgradeProxy(
      aaveYieldUsdtAddress,
      new AaveYieldV2__factory().connect(deployer),
    ) as unknown as AaveYieldV2;

    usdc = ERC20__factory.connect(usdcAddress, deployer);
    aUsdc = IAToken__factory.connect(aUsdcAddress, deployer);
    usdt = ERC20__factory.connect(usdtAddress, deployer);
    aUsdt = IAToken__factory.connect(aUsdtAddress, deployer);
  })

  it('USDC: withdraw all funds', async () => {
    console.log(`\nAaveYield USDC case. Stats:`);
    const totalSharesBefore = await aaveYieldUsdcV2.totalShares(usdc);
    const totalStakedAmountBefore = await aaveYieldUsdcV2.totalStakedAmount(usdc);
    console.log(` - totalShares: ${ethers.formatUnits(totalSharesBefore, 6)}`);
    console.log(` - totalStakedAmount: ${ethers.formatUnits(totalStakedAmountBefore, 6)} USDC`);

    for (const user of aaveYieldUsdcUsers) {
      await network.provider.request({
        method: "hardhat_impersonateAccount",
        params: [user],
      });
      const signer = await ethers.getSigner(user);
      console.log(`\nUser: ${user}`);
      await withdraw(aaveYieldUsdcV2, signer, aUsdc, usdc);
    }

    const totalSharesAfter = await aaveYieldUsdcV2.totalShares(usdc);
    const totalStakedAmountAfter = await aaveYieldUsdcV2.totalStakedAmount(usdc);
    console.log(`\nFinal AaveYield contract stats:`);
    console.log(` - totalShares: ${ethers.formatUnits(totalSharesAfter, 6)}`);
    console.log(` - totalStakedAmount: ${ethers.formatUnits(totalStakedAmountAfter, 6)} USDC`);
    expect(totalSharesAfter).to.be.equal(0n);
    expect(totalStakedAmountAfter).to.be.equal(0n);
  });

  it('USDT: withdraw all funds', async () => {
    console.log(`\nAaveYield USDT case. Stats:`);
    const totalSharesBefore = await aaveYieldUsdtV2.totalShares(usdt);
    const totalStakedAmountBefore = await aaveYieldUsdtV2.totalStakedAmount(usdt);
    console.log(` - totalShares: ${ethers.formatUnits(totalSharesBefore, 6)}`);
    console.log(` - totalStakedAmount: ${ethers.formatUnits(totalStakedAmountBefore, 6)} USDT`);

    for (const user of aaveYieldUsdtUsers) {
      await network.provider.request({
        method: "hardhat_impersonateAccount",
        params: [user],
      });
      const signer = await ethers.getSigner(user);
      console.log(`\nUser: ${user}`);
      await withdraw(aaveYieldUsdtV2, signer, aUsdt, usdt);
    }

    const totalSharesAfter = await aaveYieldUsdtV2.totalShares(usdt);
    const totalStakedAmountAfter = await aaveYieldUsdtV2.totalStakedAmount(usdt);
    console.log(`\nFinal AaveYield contract stats:`);
    console.log(` - totalShares: ${ethers.formatUnits(totalSharesAfter, 6)}`);
    console.log(` - totalStakedAmount: ${ethers.formatUnits(totalStakedAmountAfter, 6)} USDT`);
    expect(totalSharesAfter).to.be.equal(0n);
    expect(totalStakedAmountAfter).to.be.equal(0n);
  });
});
