import { expect } from 'chai';
import * as helpers from '@nomicfoundation/hardhat-network-helpers';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers, upgrades } from 'hardhat';
import { parseUnits } from 'ethers';
import { createAaveEthFork, deployAaveYieldContract } from '../shared/fixtures';
import { EthAddressData, setTokenBalance, USER_WARDEN_ADDRESS } from '../shared/utils';
import { HardhatEthersSigner } from '@nomicfoundation/hardhat-ethers/signers';
import { AaveYield, AaveYield__factory, AaveYieldUpgradeTest__factory, ERC20, IAToken } from '../../typechain-types';

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

  let availableToWithdraw = await aaveYield.getUserUnderlyingAmount(signer.address, weth9Address);
  expect(availableToWithdraw).to.be.closeTo(amount, 100);

  // check YieldStorage data
  await assertYieldStorage(amount);
}

async function withdraw(
  aaveYield: AaveYield,
  signer: HardhatEthersSigner,
  aToken: IAToken,
  tokenToWithdraw: ERC20,
  amount: bigint,
): Promise<void> {
  const aaveYieldAddress = await aaveYield.getAddress();
  const tokenAddress = await tokenToWithdraw.getAddress();

  // state before withdraw
  const aTokenScaledBalanceBefore = await aToken.scaledBalanceOf(aaveYieldAddress);
  const tokenBalanceBefore = await tokenToWithdraw.balanceOf(signer.address);

  const assertYieldStorage = await createYieldStorageAssert(aaveYield, aToken, signer.address, tokenAddress);
  const staked = await aaveYield.userStakedAmount(signer.address, tokenAddress);
  
  await aaveYield.connect(signer).unstake(tokenAddress, amount);
  const scaledBalance = await aaveYield.underlyingToShares(amount, tokenAddress);

  expect(await tokenToWithdraw.balanceOf(signer.address)).to.be.eq(amount + tokenBalanceBefore);
  expect(await aToken.scaledBalanceOf(aaveYieldAddress)).to.be.eq(aTokenScaledBalanceBefore - scaledBalance);

  // TODO remove the comment below after the YieldStorage update
  // await assertYieldStorage(-staked);
}

async function initBalance(account: string, token: ERC20, balanceStr: string): Promise<bigint> {
  const decimals = await token.decimals();
  const balance = parseUnits(balanceStr, decimals);
  await setTokenBalance(await token.getAddress(), account, balance);
  expect(await token.balanceOf(account)).to.be.eq(balance);
  return balance;
}

describe('AaveYield, deposit', () => {
  before(async () => {
    await helpers.mine();
  });

  it('1 users: stake', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH } = await loadFixture(createAaveEthFork);

    const weth9Address = await weth9.getAddress();

    // init balances
    const userInput = await initBalance(user.address, weth9, '1');

    await stake(aaveYield, user, aEthWETH, weth9, userInput, USER_WARDEN_ADDRESS);
    // await withdraw(aaveYield, user, aEthWETH, weth9, userInput);

    // expect(await aaveYield.totalStakedAmount(weth9Address)).to.be.eq(0);
    // expect(await aaveYield.totalShares(weth9Address)).to.be.eq(0);

    // expect(await aaveYield.wardenAddress(user.address)).to.be.eq(USER_WARDEN_ADDRESS);
  });

  it('2 users: stake', async () => {
    const [, user1, user2] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH } = await loadFixture(createAaveEthFork);

    const weth9Address = await weth9.getAddress();

    // init balances
    const user1Input = await initBalance(user1.address, weth9, '1');
    const user1WardenAddress = USER_WARDEN_ADDRESS;

    const user2Input = await initBalance(user2.address, weth9, '2');
    const user2WardenAddress = 'warden1233';

    await stake(aaveYield, user1, aEthWETH, weth9, user1Input, user1WardenAddress);
    await stake(aaveYield, user2, aEthWETH, weth9, user2Input, user2WardenAddress);

    await withdraw(aaveYield, user1, aEthWETH, weth9, user1Input);
    await withdraw(aaveYield, user2, aEthWETH, weth9, user2Input);

    // expect(await aaveYield.totalStakedAmount(weth9Address)).to.be.eq(0);
    // expect(await aaveYield.totalShares(weth9Address)).to.be.eq(0);

    // expect(await aaveYield.wardenAddress(user1.address)).to.be.eq(user1WardenAddress);
    // expect(await aaveYield.wardenAddress(user2.address)).to.be.eq(user2WardenAddress);
  });

  it('user stake, zero amount', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, weth9 } = await loadFixture(createAaveEthFork);
    await expect(aaveYield.connect(user).stake(weth9.target, 0, USER_WARDEN_ADDRESS)).to.be.revertedWithCustomError(
      aaveYield,
      'ZeroAmount'
    );
  });
});

describe('AaveYield, unstake', () => {
  it('unstake, not allowed token', async () => {
    const [, user] = await ethers.getSigners();
    const { owner, aaveYield, weth9, aEthWETH } = await loadFixture(createAaveEthFork);

    // init balances
    const userInput = await initBalance(user.address, weth9, '1');
    await stake(aaveYield, user, aEthWETH, weth9, userInput, USER_WARDEN_ADDRESS);
    await expect(aaveYield.unstake(aEthWETH, userInput)).to.be.revertedWithCustomError(aaveYield, 'NotAllowedToken');
  });
});

describe('AaveYield onlyOwner actions', () => {
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

  it('allowToken', async () => {
    const { owner, aaveYield } = await loadFixture(createAaveEthFork);
    expect(await aaveYield.getTokenAllowance(EthAddressData.wstEth)).to.be.false;
    await aaveYield.connect(owner).allowTokens([EthAddressData.wstEth]);
    expect(await aaveYield.getTokenAllowance(EthAddressData.wstEth)).to.be.true;
  });

  it('allowToken, notOwner', async () => {
    const { aaveYield } = await loadFixture(createAaveEthFork);
    const [_, notOwner] = await ethers.getSigners();
    await expect(aaveYield.connect(notOwner).allowTokens([EthAddressData.wstEth])).to.be.revertedWithCustomError(
      aaveYield,
      'OwnableUnauthorizedAccount'
    );
  });

  it('allowToken, already allowed', async () => {
    const { owner, aaveYield } = await loadFixture(createAaveEthFork);
    await expect(aaveYield.connect(owner).allowTokens([EthAddressData.weth])).to.be.revertedWithCustomError(
      aaveYield,
      'TokenAlreadyAllowed'
    );
  });

  it('disallowToken', async () => {
    const { owner, aaveYield } = await loadFixture(createAaveEthFork);
    expect(await aaveYield.getTokenAllowance(EthAddressData.weth)).to.be.true;
    await aaveYield.connect(owner).disallowTokens([EthAddressData.weth]);
    expect(await aaveYield.getTokenAllowance(EthAddressData.weth)).to.be.false;
  });

  it('disallowToken, notOwner', async () => {
    const { aaveYield } = await loadFixture(createAaveEthFork);
    const [_, notOwner] = await ethers.getSigners();
    await expect(aaveYield.connect(notOwner).disallowTokens([EthAddressData.weth])).to.be.revertedWithCustomError(
      aaveYield,
      'OwnableUnauthorizedAccount'
    );
  });

  it('disallowToken, already disallowed', async () => {
    const { owner, aaveYield } = await loadFixture(createAaveEthFork);
    await expect(aaveYield.connect(owner).disallowTokens([EthAddressData.wstEth])).to.be.revertedWithCustomError(
      aaveYield,
      'TokenAlreadyDisallowed'
    );
  });
});

describe('AaveYield init errors', () => {
  it('ZeroAddress', async () => {
    const [owner] = await ethers.getSigners();
    await expect(
      deployAaveYieldContract(owner, ethers.ZeroAddress, [EthAddressData.weth])
    ).to.be.revertedWithCustomError({ interface: AaveYield__factory.createInterface() }, 'ZeroAddress');
  });

  it('UnknownToken', async () => {
    const [owner] = await ethers.getSigners();
    await expect(
      deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.stEth])
    ).to.be.revertedWithCustomError({ interface: AaveYield__factory.createInterface() }, 'UnknownToken');
  });
});

describe('Aave Yield tokens', () => {
  it('1 users: stake & unstake USDT', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, usdt, aEthUsdt } = await loadFixture(createAaveEthFork);

    const usdtAddress = await usdt.getAddress();

    // init balances
    const userInput = await initBalance(user.address, usdt, '1000');

    await stake(aaveYield, user, aEthUsdt, usdt, userInput, USER_WARDEN_ADDRESS);
    await withdraw(aaveYield, user, aEthUsdt, usdt, userInput);

    // expect(await aaveYield.totalStakedAmount(usdtAddress)).to.be.eq(0);
    // expect(await aaveYield.totalShares(usdtAddress)).to.be.eq(0);

    expect(await aaveYield.wardenAddress(user.address)).to.be.eq(USER_WARDEN_ADDRESS);
  });

  it('1 users: stake & unstake USDC', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, usdc, aEthUsdc } = await loadFixture(createAaveEthFork);

    const usdcAddress = await usdc.getAddress();

    // init balances
    const userInput = await initBalance(user.address, usdc, '1000');

    await stake(aaveYield, user, aEthUsdc, usdc, userInput, USER_WARDEN_ADDRESS);
    await withdraw(aaveYield, user, aEthUsdc, usdc, userInput);

    // expect(await aaveYield.totalStakedAmount(usdcAddress)).to.be.eq(0);
    // expect(await aaveYield.totalShares(usdcAddress)).to.be.eq(0);

    expect(await aaveYield.wardenAddress(user.address)).to.be.eq(USER_WARDEN_ADDRESS);
  });
})
