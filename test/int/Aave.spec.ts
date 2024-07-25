import { expect } from 'chai';
import * as helpers from '@nomicfoundation/hardhat-network-helpers';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers, upgrades } from 'hardhat';
import { parseUnits } from 'ethers';
import {
  createAaveEthFork,
  createAaveForkWithUsdcUnderlying,
  createAaveForkWithUsdtUnderlying,
  deployAaveYieldContract,
  upgradeAaveYieldContractToV2,
} from '../shared/fixtures';
import {
  encodeStakeAction,
  encodeUnstakeAction,
  EthAddressData,
  setTokenBalance,
  WardenChain,
  WardenContractAddress,
} from '../shared/utils';
import { ActionType, CommandId } from '../shared/warden-handler-fixtures';
import { HardhatEthersSigner } from '@nomicfoundation/hardhat-ethers/signers';
import {
  AaveYield,
  AaveYieldUpgradeTest__factory,
  ERC20,
  ERC20__factory,
  IAToken,
  IERC20,
  IERC20Metadata__factory,
  IWETH9,
  TestAaveYield__factory,
  TestAxelarGateway,
} from '../../typechain-types';

async function ensureSuccessCall(aaveYield: AaveYield) {
  const [requestFailedEvent] = await aaveYield.queryFilter(aaveYield.filters.RequestFailed, -1);
  expect(requestFailedEvent).to.be.undefined;
}

async function createYieldStorageAssert(aaveYield: AaveYield, aToken: IAToken) {
  const totalSharesBefore = await aaveYield.totalShares();
  const aTokenScaledBalanceBefore = await aToken.scaledBalanceOf(await aaveYield.getAddress());

  return async (stakedDeltaExpected: bigint) => {
    const aTokenScaledBalanceDelta =
      (await aToken.scaledBalanceOf(await aaveYield.getAddress())) - aTokenScaledBalanceBefore;
    const totalShares = await aaveYield.totalShares();
    expect(totalShares).to.be.eq(totalSharesBefore + aTokenScaledBalanceDelta);
    // TODO add totalLpTokens delta check
  };
}

async function stake(
  aaveYield: AaveYield,
  signer: HardhatEthersSigner,
  aToken: IAToken,
  stakeToken: IERC20 | IWETH9,
  amount: bigint,
  stakeId: number
): Promise<bigint> {
  const aaveYieldAddress = await aaveYield.getAddress();
  const stakeTokenAddress = await stakeToken.getAddress();

  // state before stake
  const aEthScaledBalanceBefore = await aToken.scaledBalanceOf(aaveYieldAddress);
  const assertYieldStorage = await createYieldStorageAssert(aaveYield, aToken);
  const availableToWithdrawBefore = await aaveYield.sharesToUnderlying(await aaveYield.totalShares());

  // approve WETH before stake
  await stakeToken.connect(signer).transfer(aaveYield.target, amount);
  const stakePayload = encodeStakeAction(stakeId);
  const tokenSymbol = await IERC20Metadata__factory.connect(stakeTokenAddress, signer).symbol();

  // stake
  await aaveYield
    .connect(signer)
    .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, tokenSymbol, amount);
  await ensureSuccessCall(aaveYield);

  // check balances
  expect(await stakeToken.balanceOf(signer.address)).to.be.eq(0);
  expect(await stakeToken.balanceOf(aaveYieldAddress)).to.be.eq(0);
  expect(await aToken.scaledBalanceOf(aaveYieldAddress)).to.be.greaterThan(aEthScaledBalanceBefore);

  let availableToWithdraw = await aaveYield.sharesToUnderlying(await aaveYield.totalShares());
  expect(availableToWithdraw).to.be.greaterThanOrEqual(availableToWithdrawBefore + amount);

  // check YieldStorage data
  await assertYieldStorage(amount);

  const [stakeEvent] = await aaveYield.queryFilter(aaveYield.filters.Stake, -1);
  const lpAmount = stakeEvent.args[2];
  return lpAmount;
}

async function withdraw(
  aaveYield: AaveYield,
  axelarGateway: TestAxelarGateway,
  signer: HardhatEthersSigner,
  aToken: IAToken,
  tokenToWithdraw: ERC20 | IWETH9,
  lpAmount: bigint,
  unstakeId: number
): Promise<void> {
  const aaveYieldAddress = await aaveYield.getAddress();

  // state before withdraw
  const aTokenScaledBalanceBefore = await aToken.scaledBalanceOf(aaveYieldAddress);
  const tokenBalanceBefore = await tokenToWithdraw.balanceOf(axelarGateway.target);

  const unstakePayload = encodeUnstakeAction(unstakeId, lpAmount);
  await aaveYield.connect(signer).execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);
  await ensureSuccessCall(aaveYield);

  const underlyingBalance = await aaveYield.sharesToUnderlying(lpAmount);

  expect(await tokenToWithdraw.balanceOf(axelarGateway.target)).to.be.eq(underlyingBalance + tokenBalanceBefore);
  expect(await aToken.scaledBalanceOf(aaveYieldAddress)).to.be.eq(aTokenScaledBalanceBefore - lpAmount);
}

async function initBalance(account: string, token: IERC20 | IWETH9, balanceStr: string): Promise<bigint> {
  const tokenAddress = await token.getAddress();
  const erc20Token = ERC20__factory.connect(tokenAddress, ethers.provider);
  const decimals = await erc20Token.decimals();
  const balance = parseUnits(balanceStr, decimals);
  await setTokenBalance(tokenAddress, account, balance);
  expect(await token.balanceOf(account)).to.be.eq(balance);
  return balance;
}

describe('AaveYield, deposit', () => {
  before(async () => {
    await helpers.mine();
  });

  it('1 users: weth stake', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH, axelarGateway } = await loadFixture(createAaveEthFork);

    // init balances
    const userInput = await initBalance(user.address, weth9, '1');

    const stakeId = 1;
    const lpAmount = await stake(aaveYield, user, aEthWETH, weth9, userInput, stakeId);

    const unstakeId = 1;
    await withdraw(aaveYield, axelarGateway, user, aEthWETH, weth9, lpAmount, unstakeId);

    expect(await aaveYield.totalShares()).to.be.eq(0);
  });

  it('many users: weth stake', async () => {
    const [, user1, user2, user3] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH, axelarGateway } = await loadFixture(createAaveEthFork);

    for (const { user, index } of [user1, user2, user3].map((user, index) => ({ user, index }))) {
      const userInput = await initBalance(user.address, weth9, '1');

      const stakeId = index;
      const lpAmount = await stake(aaveYield, user, aEthWETH, weth9, userInput, stakeId);

      const unstakeId = index;
      await withdraw(aaveYield, axelarGateway, user, aEthWETH, weth9, lpAmount, unstakeId);
    }

    expect(await aaveYield.totalShares()).to.be.eq(0);
  });

  it('user stake, zero amount', async () => {
    const [, user1] = await ethers.getSigners();
    const { aaveYield, weth9 } = await loadFixture(createAaveEthFork);

    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await aaveYield
      .connect(user1)
      .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, await weth9.getAddress(), 0n);

    const [requestFailedEvent] = await aaveYield.queryFilter(aaveYield.filters.RequestFailed, -1);
    expect(requestFailedEvent.args[0]).to.be.eq(ActionType.Stake);
    expect(requestFailedEvent.args[1]).to.be.eq(stakeId);
    expect(requestFailedEvent.args[2]).to.be.eq(aaveYield.interface.encodeErrorResult('ZeroAmount')); // cast sig 'ZeroAmount()' = 0x1f2a2005
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
});

describe('AaveYield init errors', () => {
  it('ZeroAddress', async () => {
    const [owner] = await ethers.getSigners();
    await expect(
      deployAaveYieldContract(owner, ethers.ZeroAddress, [EthAddressData.weth])
    ).to.be.revertedWithCustomError({ interface: TestAaveYield__factory.createInterface() }, 'ZeroAddress');
  });

  it('UnknownToken', async () => {
    const [owner] = await ethers.getSigners();
    await expect(
      deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.stEth])
    ).to.be.revertedWithCustomError({ interface: TestAaveYield__factory.createInterface() }, 'UnknownToken');
  });

  it('upgrade to v2: wrong underlying token', async () => {
    const [owner] = await ethers.getSigners();

    const aaveYield = await deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.weth]);

    await expect(
      upgradeAaveYieldContractToV2(
        owner,
        aaveYield,
        EthAddressData.usdc,
        EthAddressData.axelarGateway,
        EthAddressData.axelarGasService
      )
    ).to.be.revertedWithCustomError({ interface: TestAaveYield__factory.createInterface() }, 'NotAllowedToken');
  });

  it('upgrade to v2: invalid gateway address', async () => {
    const [owner] = await ethers.getSigners();

    const aaveYield = await deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.weth]);

    await expect(
      upgradeAaveYieldContractToV2(
        owner,
        aaveYield,
        EthAddressData.weth,
        ethers.ZeroAddress,
        EthAddressData.axelarGasService
      )
    ).to.be.revertedWithCustomError({ interface: TestAaveYield__factory.createInterface() }, 'InvalidAddress');
  });

  it('upgrade to v2: invalid gas service address', async () => {
    const [owner] = await ethers.getSigners();

    const aaveYield = await deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.weth]);

    await expect(
      deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.stEth])
    ).to.be.revertedWithCustomError({ interface: TestAaveYield__factory.createInterface() }, 'UnknownToken');
  });

  it('upgrade to v2: wrong underlying token', async () => {
    const [owner] = await ethers.getSigners();

    const aaveYield = await deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.weth]);

    await expect(
      upgradeAaveYieldContractToV2(
        owner,
        aaveYield,
        EthAddressData.usdc,
        EthAddressData.axelarGateway,
        EthAddressData.axelarGasService
      )
    ).to.be.revertedWithCustomError({ interface: TestAaveYield__factory.createInterface() }, 'NotAllowedToken');
  });

  it('upgrade to v2: invalid gateway address', async () => {
    const [owner] = await ethers.getSigners();

    const aaveYield = await deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.weth]);

    await expect(
      upgradeAaveYieldContractToV2(
        owner,
        aaveYield,
        EthAddressData.weth,
        ethers.ZeroAddress,
        EthAddressData.axelarGasService
      )
    ).to.be.revertedWithCustomError({ interface: TestAaveYield__factory.createInterface() }, 'InvalidAddress');
  });

  it('upgrade to v2: invalid gas service address', async () => {
    const [owner] = await ethers.getSigners();

    const aaveYield = await deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.weth]);

    await expect(
      upgradeAaveYieldContractToV2(
        owner,
        aaveYield,
        EthAddressData.weth,
        EthAddressData.axelarGateway,
        ethers.ZeroAddress
      )
    ).to.be.revertedWithCustomError({ interface: TestAaveYield__factory.createInterface() }, 'InvalidAddress');
  });
});

describe('Aave Yield tokens', () => {
  it('1 users: stake & unstake USDT', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, usdt, aEthUsdt, axelarGateway } = await loadFixture(createAaveForkWithUsdtUnderlying);

    // init balances
    const userInput = await initBalance(user.address, usdt, '1000');
    expect(await usdt.balanceOf(user.address)).to.be.eq(userInput);

    const stakeId = 1;
    const lpAmount = await stake(aaveYield, user, aEthUsdt, usdt, userInput, stakeId);
    const unstakeId = 1;
    await withdraw(aaveYield, axelarGateway, user, aEthUsdt, usdt, lpAmount, unstakeId);
  });

  it('1 users: stake & unstake USDC', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, usdc, aEthUsdc, axelarGateway } = await loadFixture(createAaveForkWithUsdcUnderlying);

    // init balances
    const userInput = await initBalance(user.address, usdc, '1000');

    const stakeId = 1;
    const lpAmount = await stake(aaveYield, user, aEthUsdc, usdc, userInput, stakeId);

    const unstakeId = 1;
    await withdraw(aaveYield, axelarGateway, user, aEthUsdc, usdc, lpAmount, unstakeId);
  });
});
