import { expect } from 'chai';
import * as helpers from '@nomicfoundation/hardhat-network-helpers';
import { loadFixture, mine, time } from '@nomicfoundation/hardhat-network-helpers';
import { ethers, upgrades } from 'hardhat';
import { parseEther, parseUnits } from 'ethers';
import {
  createAaveEthFork,
  createAaveForkWithUsdcUnderlying,
  createAaveForkWithUsdtUnderlying,
  deployAaveYieldContract,
  upgradeAaveYieldContractToV2,
} from '../shared/fixtures';
import {
  decodeWardenStakeResponse,
  decodeWardenUnstakeResponse,
  encodeStakeAction,
  encodeUnstakeAction,
  EthAddressData,
  setTokenBalance,
  WardenChain,
  WardenContractAddress,
} from '../shared/utils';
import { ActionType, CommandId, Status } from '../shared/warden-handler-fixtures';
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
  const totalLpBefore = await aaveYield.totalLpTokens();
  const aTokenScaledBalanceBefore = await aToken.scaledBalanceOf(await aaveYield.getAddress());

  return async (lpDelta: bigint) => {
    const aTokenScaledBalanceDelta =
      (await aToken.scaledBalanceOf(await aaveYield.getAddress())) - aTokenScaledBalanceBefore;

    const totalShares = await aaveYield.totalShares();
    expect(totalShares).to.be.eq(totalSharesBefore + aTokenScaledBalanceDelta);

    const totalLp = await aaveYield.totalLpTokens();
    expect(totalLp).to.be.eq(totalLpBefore + lpDelta);
  };
}

async function stake(
  aaveYield: AaveYield,
  axelarGateway: TestAxelarGateway,
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

  // adding some blocks and resetting the next block timestamp
  // so the aave shares to underlying conversion occurred with the same timestamp in both view query and tx
  await mine(1000);
  const currentBlockTimestamp = (await signer.provider.getBlock(await signer.provider.getBlockNumber()))?.timestamp!;
  await time.setNextBlockTimestamp(currentBlockTimestamp);

  const expectedLpAmount = await aaveYield.underlyingToLp(amount);
  const lpBefore = await aaveYield.totalLpTokens();
  const availableToWithdrawBefore = await aaveYield.lpToUnderlying(await aaveYield.totalLpTokens());

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
  expect(await aaveYield.totalLpTokens()).to.be.eq(expectedLpAmount + lpBefore);

  let availableToWithdraw = await aaveYield.lpToUnderlying(await aaveYield.totalLpTokens());
  expect(availableToWithdraw).to.be.closeTo(availableToWithdrawBefore + amount, 1);

  const [stakeEvent] = await aaveYield.queryFilter(aaveYield.filters.Stake, -1);
  const lpAmount = stakeEvent.args[2];

  // check YieldStorage data
  await assertYieldStorage(lpAmount);

  const [axelarGatewayEvent] = await axelarGateway.queryFilter(axelarGateway.filters.ContractCall, -1);
  expect(axelarGatewayEvent.args[0]).to.be.eq(aaveYield.target);
  expect(axelarGatewayEvent.args[1]).to.be.eq(WardenChain);
  expect(axelarGatewayEvent.args[2]).to.be.eq(WardenContractAddress);

  const stakeResponse = decodeWardenStakeResponse(axelarGatewayEvent.args[4]);
  expect(stakeResponse.actionType).to.be.eq(ActionType.Stake);
  expect(stakeResponse.status).to.be.eq(Status.Success);
  expect(stakeResponse.lpAmount).to.be.closeTo(lpAmount, 1);
  expect(stakeResponse.actionId).to.be.eq(stakeId);

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
  const assertYieldStorage = await createYieldStorageAssert(aaveYield, aToken);
  const tokenBalanceBefore = await tokenToWithdraw.balanceOf(axelarGateway.target);

  // adding some blocks and resetting the next block timestamp
  // so the aave shares to underlying conversion occurred with the same timestamp in both view query and tx
  await mine(1000);
  const currentBlockTimestamp = (await signer.provider.getBlock(await signer.provider.getBlockNumber()))?.timestamp!;
  await time.setNextBlockTimestamp(currentBlockTimestamp);

  const lpUnderlyingBalance = await aaveYield.lpToUnderlying(lpAmount);

  const unstakePayload = encodeUnstakeAction(unstakeId, lpAmount);
  await aaveYield.connect(signer).execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);
  await ensureSuccessCall(aaveYield);

  const expectedBalance = lpUnderlyingBalance + tokenBalanceBefore;

  // closeTo is to avoid underlying balance calculation precision issue
  expect(await tokenToWithdraw.balanceOf(axelarGateway.target)).to.be.closeTo(expectedBalance, 1);
  expect(await aToken.scaledBalanceOf(aaveYieldAddress)).to.be.lt(aTokenScaledBalanceBefore);

  // check YieldStorage data
  await assertYieldStorage(-lpAmount);

  const [axelarGatewayEvent] = await axelarGateway.queryFilter(axelarGateway.filters.ContractCallWithToken, -1);
  expect(axelarGatewayEvent.args[0]).to.be.eq(aaveYield.target);
  expect(axelarGatewayEvent.args[1]).to.be.eq(WardenChain);
  expect(axelarGatewayEvent.args[2]).to.be.eq(WardenContractAddress);
  expect(axelarGatewayEvent.args[5]).to.be.eq('symbol' in tokenToWithdraw ? await tokenToWithdraw.symbol() : 'WETH');
  expect(axelarGatewayEvent.args[6]).to.be.eq(lpUnderlyingBalance);

  const unstakeResponse = decodeWardenUnstakeResponse(axelarGatewayEvent.args[4]);
  expect(unstakeResponse.actionType).to.be.eq(ActionType.Unstake);
  expect(unstakeResponse.status).to.be.eq(Status.Success);
  expect(unstakeResponse.reinitUnstakeId).to.be.eq(unstakeId);
  expect(unstakeResponse.actionId).to.be.eq(unstakeId);
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

describe('AaveYield', () => {
  before(async () => {
    await helpers.mine();
  });

  it('1 users: weth stake', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH, axelarGateway } = await loadFixture(createAaveEthFork);

    // init balances
    const userInput = await initBalance(user.address, weth9, '1');

    const stakeId = 1;
    const lpAmount = await stake(aaveYield, axelarGateway, user, aEthWETH, weth9, userInput, stakeId);

    const unstakeId = 1;
    await withdraw(aaveYield, axelarGateway, user, aEthWETH, weth9, lpAmount, unstakeId);

    expect(await aaveYield.totalLpTokens()).to.be.eq(0);
    expect(await aaveYield.totalShares()).to.be.eq(0);
  });

  it('many users: weth stake', async () => {
    const [, user1, user2, user3] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH, axelarGateway } = await loadFixture(createAaveEthFork);

    const users = [user1, user2, user3];
    const lpAmounts = new Array(users.length);

    for (const { user, index } of users.map((user, index) => ({ user, index }))) {
      const userInput = await initBalance(user.address, weth9, '1');

      const stakeId = index;
      const lpAmount = await stake(aaveYield, axelarGateway, user, aEthWETH, weth9, userInput, stakeId);
      lpAmounts[index] = lpAmount;
    }

    for (const { user, index } of users.map((user, index) => ({ user, index }))) {
      const unstakeId = index;
      await withdraw(aaveYield, axelarGateway, user, aEthWETH, weth9, lpAmounts[index], unstakeId);
    }

    expect(await aaveYield.totalLpTokens()).to.be.eq(0);
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

  it('user unstake, zero amount', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, weth9, aEthWETH, axelarGateway } = await loadFixture(createAaveEthFork);

    // init balances
    const userInput = await initBalance(user.address, weth9, '1');

    const stakeId = 1;
    await stake(aaveYield, axelarGateway, user, aEthWETH, weth9, userInput, stakeId);

    const unstakeId = 1;
    const unstakePayload = encodeUnstakeAction(unstakeId, 0n);
    await aaveYield.connect(user).execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);

    const [requestFailedEvent] = await aaveYield.queryFilter(aaveYield.filters.RequestFailed, -1);
    expect(requestFailedEvent.args[0]).to.be.eq(ActionType.Unstake);
    expect(requestFailedEvent.args[1]).to.be.eq(stakeId);
    expect(requestFailedEvent.args[2]).to.be.eq(aaveYield.interface.encodeErrorResult('ZeroAmount')); // cast sig 'ZeroAmount()' = 0x1f2a2005
  });
});

describe('lp calculations', () => {
  it('empty pool', async () => {
    const { aaveYield } = await loadFixture(createAaveEthFork);

    // checks to ensure that pool is empty
    expect(await aaveYield.totalLpTokens()).to.be.eq(0);
    expect(await aaveYield.totalShares()).to.be.eq(0);

    const underlyingAmount = parseUnits('1', 18);
    expect(await aaveYield.lpToUnderlying(underlyingAmount)).to.be.eq(0);
    expect(await aaveYield.underlyingToLp(underlyingAmount)).to.be.eq(underlyingAmount);
  });

  it('not empty pool', async () => {
    const { aaveYield, weth9, aEthWETH, axelarGateway } = await loadFixture(createAaveEthFork);
    const [_, user] = await ethers.getSigners();

    // init balances
    const userInput = await initBalance(user.address, weth9, '1');

    const stakeId = 1;
    await stake(aaveYield, axelarGateway, user, aEthWETH, weth9, userInput, stakeId);

    const totalLp = await aaveYield.totalLpTokens();
    await mine(1000);
    const totalUnderlying = await aEthWETH.balanceOf(aaveYield.target);

    const lpAmount = totalLp / 2n;
    const expectedUnderlyingForLp = (lpAmount * totalUnderlying) / totalLp;
    expect(await aaveYield.lpToUnderlying(lpAmount)).to.be.closeTo(expectedUnderlyingForLp, 1);

    const underlyingAmount = totalUnderlying / 4n;
    const expectedLp = (totalLp * underlyingAmount) / totalUnderlying;
    expect(await aaveYield.underlyingToLp(underlyingAmount)).to.be.closeTo(expectedLp, 1);
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
    const lpAmount = await stake(aaveYield, axelarGateway, user, aEthUsdt, usdt, userInput, stakeId);
    const unstakeId = 1;
    await withdraw(aaveYield, axelarGateway, user, aEthUsdt, usdt, lpAmount, unstakeId);
  });

  it('1 users: stake & unstake USDC', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield, usdc, aEthUsdc, axelarGateway } = await loadFixture(createAaveForkWithUsdcUnderlying);

    // init balances
    const userInput = await initBalance(user.address, usdc, '1000');

    const stakeId = 1;
    const lpAmount = await stake(aaveYield, axelarGateway, user, aEthUsdc, usdc, userInput, stakeId);

    const unstakeId = 1;
    await withdraw(aaveYield, axelarGateway, user, aEthUsdc, usdc, lpAmount, unstakeId);
  });
});

describe('Aave errors', () => {
  it('reinit forbidden', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield } = await loadFixture(createAaveForkWithUsdtUnderlying);

    await expect(aaveYield.connect(user).executeReinit()).to.be.revertedWith('Not supported');
  });

  it('stake can be called by axelar only', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield } = await loadFixture(createAaveForkWithUsdtUnderlying);
    await expect(aaveYield.connect(user).stake(1, parseUnits('100', 6))).to.be.revertedWithoutReason();
  });

  it('unstake can be called by axelar only', async () => {
    const [, user] = await ethers.getSigners();
    const { aaveYield } = await loadFixture(createAaveForkWithUsdtUnderlying);
    await expect(aaveYield.connect(user).unstake(1, parseUnits('100', 6))).to.be.revertedWithoutReason();
  });
});
