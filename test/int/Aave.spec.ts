import { expect } from 'chai';
import * as helpers from '@nomicfoundation/hardhat-network-helpers';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers, upgrades } from 'hardhat';
import { parseEther } from 'ethers';
import { createAaveEthFork } from '../shared/fixtures';
import { USER_WARDEN_ADDRESS, setTokenBalance } from '../shared/utils';
import { HardhatEthersSigner } from '@nomicfoundation/hardhat-ethers/signers';
import { AaveYield, AaveYieldUpgradeTest__factory, ERC20, IAToken, IERC20 } from '../../typechain-types';

async function createYieldStorageAssert(aaveYield: AaveYield, aToken: IAToken, account: string, token: string) {
  const stakedAmountBefore = await aaveYield.userStakedAmount(account, token);
  const totalStakedAmountBefore = await aaveYield.totalStakedAmount(token);
  const sharesBefore = await aaveYield.userShares(account, token);
  const totalSharesBefore = await aaveYield.totalShares(token);
  const aTokenScaledBalanceBefore = await aToken.scaledBalanceOf(await aaveYield.getAddress());

  return async (stakedDeltaExpected: bigint) => {
    const stakedAmount = await aaveYield.userStakedAmount(account, token);
    const totalStakedAmount = await aaveYield.totalStakedAmount(token);
    expect(stakedAmount).to.be.eq(stakedAmountBefore + stakedDeltaExpected);
    expect(totalStakedAmount).to.be.eq(totalStakedAmountBefore + stakedDeltaExpected);

    const shares = await aaveYield.userShares(account, token);
    const totalShares = await aaveYield.totalShares(token);

    const aTokenScaledBalanceDelta =
      (await aToken.scaledBalanceOf(await aaveYield.getAddress())) - aTokenScaledBalanceBefore;

    expect(shares).to.be.eq(sharesBefore + aTokenScaledBalanceDelta);
    expect(totalShares).to.be.eq(totalSharesBefore + aTokenScaledBalanceDelta);
  };
}

async function stake(
  aaveYield: AaveYield,
  signer: HardhatEthersSigner,
  aEthWETH: IAToken,
  weth9: ERC20,
  amount: bigint,
  userWardenAddress: string
): Promise<void> {
  const aaveYieldAddress = await aaveYield.getAddress();
  const weth9Address = await weth9.getAddress();

  // state before stake
  const aEthScaledBalanceBefore = await aEthWETH.scaledBalanceOf(aaveYieldAddress);
  const assertYieldStorage = await createYieldStorageAssert(aaveYield, aEthWETH, signer.address, weth9Address);

  // approve WETH before stake
  await weth9.connect(signer).approve(aaveYieldAddress, amount);
  // stake
  await aaveYield.connect(signer).stake(weth9Address, amount, userWardenAddress);

  // check balances
  expect(await weth9.balanceOf(signer.address)).to.be.eq(0);
  expect(await weth9.balanceOf(aaveYieldAddress)).to.be.eq(0);
  expect(await aEthWETH.scaledBalanceOf(aaveYieldAddress)).to.be.greaterThan(aEthScaledBalanceBefore);

  let availableToWithdraw = await aaveYield.getAvailableToWithdraw(signer.address, weth9Address);
  expect(availableToWithdraw).to.be.eq(amount);

  // check YieldStorage data
  await assertYieldStorage(amount);
}

async function withdraw(
  aaveYield: AaveYield,
  signer: HardhatEthersSigner,
  aEthWETH: IAToken,
  weth9: ERC20
): Promise<void> {
  const aaveYieldAddress = await aaveYield.getAddress();
  const weth9Address = await weth9.getAddress();

  // state before withdraw
  const aEthScaledBalanceBefore = await aEthWETH.scaledBalanceOf(aaveYieldAddress);
  const assertYieldStorage = await createYieldStorageAssert(aaveYield, aEthWETH, signer.address, weth9Address);
  const availableToWithdraw = await aaveYield.getAvailableToWithdraw(signer.address, weth9Address);
  const staked = await aaveYield.userStakedAmount(signer.address, weth9Address);
  const scaledBalance = await aaveYield.userShares(signer.address, weth9Address);

  await aaveYield.connect(signer).withdraw(weth9);
  expect(await weth9.balanceOf(signer.address)).to.be.greaterThanOrEqual(availableToWithdraw);
  expect(await aEthWETH.scaledBalanceOf(aaveYieldAddress)).to.be.eq(aEthScaledBalanceBefore - scaledBalance);

  await assertYieldStorage(-staked);
}

async function initBalance(account: string, token: IERC20, ethers: string): Promise<bigint> {
  const balance = parseEther(ethers);
  await setTokenBalance(await token.getAddress(), account, balance);
  expect(await token.balanceOf(account)).to.be.eq(balance);
  return balance;
}

describe('AaveYield', () => {
  before(async () => {
    await helpers.mine();
  });

  it('1 users: stake & unstake', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH } = await loadFixture(createAaveEthFork);

    await aaveYield.enableWithdrawals();
    const weth9Address = await weth9.getAddress();

    // init balances
    const userInput = await initBalance(user.address, weth9, '1');
    console.log(`User init balance: ${ethers.formatEther(userInput)} WETH`);

    console.log(`User stake`);
    await stake(aaveYield, user, aEthWETH, weth9, userInput, USER_WARDEN_ADDRESS);

    console.log(`User withdraw`);
    await withdraw(aaveYield, user, aEthWETH, weth9);

    expect(await aaveYield.totalStakedAmount(weth9Address)).to.be.eq(0);
    expect(await aaveYield.totalShares(weth9Address)).to.be.eq(0);

    expect(await aaveYield.wardenAddress(user.address)).to.be.eq(USER_WARDEN_ADDRESS);
  });

  it('2 users: stake & unstake', async () => {
    const [, user1, user2] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH } = await loadFixture(createAaveEthFork);

    await aaveYield.enableWithdrawals();
    const weth9Address = await weth9.getAddress();

    // init balances
    const user1Input = await initBalance(user1.address, weth9, '1');
    const user1WardenAddress = USER_WARDEN_ADDRESS;
    console.log(`User1 init balance: ${ethers.formatEther(user1Input)} WETH`);

    const user2Input = await initBalance(user2.address, weth9, '2');
    const user2WardenAddress = 'warden1233';
    console.log(`User2 init balance: ${ethers.formatEther(user2Input)} WETH`);

    console.log(`User1 stake`);
    await stake(aaveYield, user1, aEthWETH, weth9, user1Input, user1WardenAddress);

    console.log(`User2 stake`);
    await stake(aaveYield, user2, aEthWETH, weth9, user2Input, user2WardenAddress);
    console.log(`User1 withdraw`);
    await withdraw(aaveYield, user1, aEthWETH, weth9);

    console.log(`User2 withdraw`);
    await withdraw(aaveYield, user2, aEthWETH, weth9);

    expect(await aaveYield.totalStakedAmount(weth9Address)).to.be.eq(0);
    expect(await aaveYield.totalShares(weth9Address)).to.be.eq(0);

    expect(await aaveYield.wardenAddress(user1.address)).to.be.eq(user1WardenAddress);
    expect(await aaveYield.wardenAddress(user2.address)).to.be.eq(user2WardenAddress);
  });
});

describe('onlyOwner actions', () => {
  it('authorizeUpgrade', async () => {
    const { owner, aaveYield } = await loadFixture(createAaveEthFork);
    expect(function () {
      aaveYield.interface.getFunctionName('upgradedTest');
    }).to.throw(TypeError);

    const aaveYieldV2 = await upgrades.upgradeProxy(
      aaveYield.target,
      await new AaveYieldUpgradeTest__factory().connect(owner)
    );

    expect(await aaveYieldV2.upgradedTest()).to.be.true;
  });

  it('authorizeUpgrade, not owner', async () => {
    const { owner, aaveYield } = await loadFixture(createAaveEthFork);
    const [_, user] = await ethers.getSigners();
    expect(user.address).to.be.not.eq(owner.address);
    await expect(
      upgrades.upgradeProxy(aaveYield.target, await new AaveYieldUpgradeTest__factory().connect(user))
    ).to.be.revertedWithCustomError(aaveYield, 'OwnableUnauthorizedAccount');
  });
});
