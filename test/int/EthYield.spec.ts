import { expect } from 'chai';
import { loadFixture, mine } from '@nomicfoundation/hardhat-network-helpers';
import { createEthYieldFork, deployEthYieldContract } from '../shared/fixtures';
import { ethers, upgrades } from 'hardhat';
import { parseEther, parseUnits } from 'ethers';
import {
  EthAddressData,
  WardenChain,
  WardenContractAddress,
  decodeWardenReinitResponse,
  decodeWardenStakeResponse,
  decodeWardenUnstakeResponse,
  encodeStakeAction,
  encodeUnstakeAction,
  finalizeLidoWithdraw,
} from '../shared/utils';
import { EthYield, EthYieldUpgradeTest__factory, TestEthYield__factory } from '../../typechain-types';
import { ActionType, CommandId, Status } from '../shared/warden-handler-fixtures';

async function ensureSuccessCall(ethYield: EthYield) {
  const [requestFailedEvent] = await ethYield.queryFilter(ethYield.filters.RequestFailed, -1);
  expect(requestFailedEvent).to.be.undefined;
}

describe('EthYield stake', () => {
  it('user stake, one', async () => {
    const {
      eigenLayerDelegationManager,
      eigenLayerOperator,
      eigenLayerStrategy,
      ethYield,
      weth9,
      stEth,
      axelarGateway,
    } = await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const userEthBalanceBefore = await user.provider.getBalance(user.address);
    const eigenLayerFilter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;
    const axelarGatewayFilter = axelarGateway.filters.ContractCall;

    // here we simulate stake request coming from Axelar gateway
    const input = parseEther('1');
    await weth9.connect(user).deposit({ value: input });
    await weth9.connect(user).transfer(ethYield.target, input);
    const stakeId = 1;

    const lpTokenAmount = await ethYield.underlyingToLp(input);

    const stakePayload = encodeStakeAction(stakeId);
    await ethYield.executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', input);
    await ensureSuccessCall(ethYield);

    const userEthBalanceAfter = await user.provider.getBalance(user.address);
    expect(userEthBalanceBefore - userEthBalanceAfter).to.be.gte(input);

    const contractShares = await eigenLayerStrategy.shares(ethYield.target);
    expect(contractShares).to.be.eq(await ethYield.totalShares());

    expect(lpTokenAmount).to.be.eq(await ethYield.totalLpTokens());
    expect(await stEth.balanceOf(ethYield.target)).to.be.lessThanOrEqual(1);

    const [eigenLayerEvent] = await eigenLayerDelegationManager.queryFilter(eigenLayerFilter, -1);
    expect(eigenLayerEvent.args[0]).to.be.eq(eigenLayerOperator);
    expect(eigenLayerEvent.args[1]).to.be.eq(ethYield.target);
    expect(eigenLayerEvent.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(eigenLayerEvent.args[3]).to.be.eq(contractShares);

    const [axelarGatewayEvent] = await axelarGateway.queryFilter(axelarGatewayFilter, -1);
    expect(axelarGatewayEvent.args[0]).to.be.eq(ethYield.target);
    expect(axelarGatewayEvent.args[1]).to.be.eq(WardenChain);
    expect(axelarGatewayEvent.args[2]).to.be.eq(WardenContractAddress);

    const stakeResponse = decodeWardenStakeResponse(axelarGatewayEvent.args[4]);
    expect(stakeResponse.actionType).to.be.eq(ActionType.Stake);
    expect(stakeResponse.status).to.be.eq(Status.Success);
    expect(stakeResponse.lpAmount).to.be.eq(lpTokenAmount);
    expect(stakeResponse.actionId).to.be.eq(stakeId);
  });

  it('user stake, many', async () => {
    const {
      eigenLayerDelegationManager,
      eigenLayerOperator,
      eigenLayerStrategy,
      ethYield,
      weth9,
      stEth,
      axelarGateway,
    } = await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);
    const users = (await ethers.getSigners()).slice(1, 3);
    const axelarGatewayFilter = axelarGateway.filters.ContractCall;

    for (const { user, index } of users.map((user, index) => ({ user, index: index + 1 }))) {
      const userEthBalanceBefore = await ethers.provider.getBalance(user.address);

      // here we simulate stake request coming from Axelar gateway
      const input = parseEther(index.toString());
      await weth9.connect(user).deposit({ value: input });
      await weth9.connect(user).transfer(ethYield.target, input);
      const stakeId = index;

      const lpTokenAmount = await ethYield.underlyingToLp(input);
      const totalLpBefore = await ethYield.totalLpTokens();
      const stEthBalanceBefore = await stEth.balanceOf(ethYield.target);

      const stakePayload = encodeStakeAction(stakeId);
      await ethYield.executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', input);
      await ensureSuccessCall(ethYield);

      const userEthBalanceAfter = await user.provider.getBalance(user.address);
      expect(userEthBalanceBefore - userEthBalanceAfter).to.be.gte(input);

      const contractShares = await eigenLayerStrategy.shares(ethYield.target);
      expect(contractShares).to.be.eq(await ethYield.totalShares());

      const totalLpAfter = await ethYield.totalLpTokens();
      expect(totalLpAfter).to.be.closeTo(lpTokenAmount + totalLpBefore, 1);

      // small stEth calculation discrepancies
      const stEthBalanceAfter = await stEth.balanceOf(ethYield.target);
      expect(stEthBalanceAfter - stEthBalanceBefore).to.be.lessThanOrEqual(1);

      const [axelarGatewayEvent] = await axelarGateway.queryFilter(axelarGatewayFilter, -1);
      expect(axelarGatewayEvent.args[0]).to.be.eq(ethYield.target);
      expect(axelarGatewayEvent.args[1]).to.be.eq(WardenChain);
      expect(axelarGatewayEvent.args[2]).to.be.eq(WardenContractAddress);

      const stakeResponse = decodeWardenStakeResponse(axelarGatewayEvent.args[4]);
      expect(stakeResponse.actionType).to.be.eq(ActionType.Stake);
      expect(stakeResponse.status).to.be.eq(Status.Success);
      expect(stakeResponse.lpAmount).to.be.closeTo(lpTokenAmount, 1);
      expect(stakeResponse.actionId).to.be.eq(stakeId);
    }
  });

  it('user stake, zero amount', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, ethYield } = await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);

    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await ethYield.executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', 0);

    const [event] = await ethYield.queryFilter(ethYield.filters.RequestFailed, -1);

    expect(event.args[2]).to.be.eq(ethYield.interface.encodeErrorResult('ZeroAmount'));
    expect(event.args[1]).to.be.eq(stakeId);
    expect(event.args[0]).to.be.eq(ActionType.Stake);
  });
});

describe('EthYield withdraw', () => {
  it('user full withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue, weth9, axelarGateway } =
      await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const stakeAmount = parseEther('1');
    await weth9.connect(user).deposit({ value: stakeAmount });
    await weth9.connect(user).transfer(ethYield.target, stakeAmount);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await ethYield.executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', stakeAmount);
    await ensureSuccessCall(ethYield);

    const elWithdrawFilter = ethYield.filters.EigenLayerWithdrawStart;
    const lidoWithdrawFilter = ethYield.filters.LidoWithdrawStart;
    const axelarGatewayWithTokenFilter = axelarGateway.filters.ContractCallWithToken;

    const totalSharesBefore = await ethYield.totalShares();
    const totalLpBefore = await ethYield.totalLpTokens();
    const lpToUnstake = totalLpBefore / 2n;

    const unstakeId = 2;
    const unstakePayload = encodeUnstakeAction(unstakeId, lpToUnstake);
    const expectedOutput = await ethYield.lpToUnderlying(lpToUnstake);

    await ethYield.execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);
    await ensureSuccessCall(ethYield);

    const [elWithdrawStartEvent] = await ethYield.queryFilter(elWithdrawFilter, -1);
    const elElement = await ethYield.getEigenLayerWithdrawalQueueElement(0);
    expect(elElement.unstakeId).to.be.eq(elWithdrawStartEvent.args[0]);
    expect(elElement.shares).to.be.eq(elWithdrawStartEvent.args[1]);
    expect(elElement.blockNumber).to.be.eq(elWithdrawStartEvent.blockNumber);

    const unstakeResponse = decodeWardenUnstakeResponse(await axelarGateway.callContractPayload());
    expect(unstakeResponse.actionType).to.be.eq(ActionType.Unstake);
    expect(unstakeResponse.status).to.be.eq(Status.Success);
    expect(unstakeResponse.reinitUnstakeId).to.be.eq(0);
    expect(unstakeResponse.actionId).to.be.eq(unstakeId);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    await ethYield.connect(user).executeReinit();
    await ensureSuccessCall(ethYield);

    const [lidoWithdrawStartEvent] = await ethYield.queryFilter(lidoWithdrawFilter, -1);
    const lidoElement = await ethYield.getLidoWithdrawalQueueElement(0);
    expect(lidoElement.requestId).to.be.eq(await lidoWithdrawalQueue.getLastRequestId());
    expect(lidoElement.requested).to.be.eq(lidoWithdrawStartEvent.args[1]);

    const balanceBefore = await user.provider.getBalance(ethYield.target);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, lidoElement.requestId);
    await ethYield.connect(user).executeReinit();

    await ethYield.connect(user).executeReinit();
    await ensureSuccessCall(ethYield);

    const [lidoWithdrawComplete] = await ethYield.queryFilter(ethYield.filters.LidoWithdrawComplete, -1);
    expect(lidoWithdrawComplete.args[0]).to.be.eq(unstakeId);
    expect(lidoWithdrawComplete.args[1]).to.be.eq(lidoElement.requested);

    const balanceAfter = await weth9.balanceOf(axelarGateway.target);
    expect(balanceAfter).to.be.eq(balanceBefore + lidoElement.requested);

    expect(await ethYield.totalShares()).to.be.lt(totalSharesBefore);
    expect(await ethYield.totalLpTokens()).to.be.eq(totalLpBefore - lpToUnstake);

    const [axelarGatewayWithdrawEvent] = await axelarGateway.queryFilter(axelarGatewayWithTokenFilter, -1);
    expect(axelarGatewayWithdrawEvent.args[0]).to.be.eq(ethYield.target);
    expect(axelarGatewayWithdrawEvent.args[1]).to.be.eq(WardenChain);
    expect(axelarGatewayWithdrawEvent.args[2]).to.be.eq(WardenContractAddress);
    expect(axelarGatewayWithdrawEvent.args[5]).to.be.eq('WETH');
    expect(axelarGatewayWithdrawEvent.args[6]).to.be.eq(expectedOutput);

    const reinitResponse = decodeWardenReinitResponse(axelarGatewayWithdrawEvent.args[4]);
    expect(reinitResponse.actionType).to.be.eq(ActionType.Reinit);
    expect(reinitResponse.reinitUnstakeId).to.be.eq(unstakeId);
  });

  it('too low withdraw', async () => {
    const { ethYield, lidoWithdrawalQueue, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const stakeAmount = parseEther('1');
    await weth9.connect(user).deposit({ value: stakeAmount });
    await weth9.connect(user).transfer(ethYield.target, stakeAmount);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await ethYield
      .connect(user)
      .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', stakeAmount);

    const minLp = await ethYield.underlyingToLp(await lidoWithdrawalQueue.MIN_STETH_WITHDRAWAL_AMOUNT());
    const unstakeId = 1;
    const unstakePayload = encodeUnstakeAction(unstakeId, minLp);
    await ethYield.connect(user).execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);

    const [requestFailed] = await ethYield.queryFilter(ethYield.filters.RequestFailed, -1);
    expect(requestFailed.args[0]).to.be.eq(ActionType.Unstake);
    expect(requestFailed.args[1]).to.be.eq(unstakeId);
    // cast sig "LowWithdrawalAmount(uint256)" = 0x9d7ecf5d
    expect(requestFailed.args[2].startsWith('0x9d7ecf5d')).to.be.true;
  });

  it('lowest allowed unstake passes', async () => {
    const { ethYield, lidoWithdrawalQueue, eigenLayerDelegationManager, weth9, axelarGateway } =
      await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const stakeAmount = parseEther('1');
    await weth9.connect(user).deposit({ value: stakeAmount });
    await weth9.connect(user).transfer(ethYield.target, stakeAmount);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await ethYield
      .connect(user)
      .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', stakeAmount);
    await ensureSuccessCall(ethYield);

    const minAllowedLp = (await ethYield.underlyingToLp(await lidoWithdrawalQueue.MIN_STETH_WITHDRAWAL_AMOUNT())) + 2n;

    const unstakeId = 1;
    const unstakePayload = encodeUnstakeAction(unstakeId, minAllowedLp);
    await ethYield.connect(user).execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);
    await ensureSuccessCall(ethYield);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user).executeReinit();
    await ensureSuccessCall(ethYield);

    const lidoElement = await ethYield.getLidoWithdrawalQueueElement(0);
    const balanceBefore = await weth9.balanceOf(axelarGateway.target);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, lidoElement.requestId);
    await ethYield.connect(user).executeReinit();
    await ensureSuccessCall(ethYield);

    const balanceAfter = await weth9.balanceOf(axelarGateway.target);
    expect(balanceAfter).to.be.eq(balanceBefore + lidoElement.requested);
  });
});

describe('lp calculations', () => {
  it('empty pool', async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);

    // checks to ensure that pool is empty
    expect(await ethYield.totalLpTokens()).to.be.eq(0);
    expect(await ethYield.totalShares()).to.be.eq(0);

    const underlyingAmount = parseUnits('1', 18);
    expect(await ethYield.lpToUnderlying(underlyingAmount)).to.be.eq(0);
    expect(await ethYield.underlyingToLp(underlyingAmount)).to.be.eq(underlyingAmount);
  });

  it('not empty pool', async () => {
    const { ethYield, weth9, eigenLayerStrategy } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const stakeAmount = parseEther('1');
    await weth9.connect(user).deposit({ value: stakeAmount });
    await weth9.connect(user).transfer(ethYield.target, stakeAmount);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await ethYield
      .connect(user)
      .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', stakeAmount);
    await ensureSuccessCall(ethYield);

    const totalShares = await ethYield.totalShares();
    const totalLp = await ethYield.totalLpTokens();
    await mine(1000);
    const totalUnderlying = await eigenLayerStrategy.sharesToUnderlyingView(totalShares);

    const lpAmount = totalLp / 2n;
    const expectedUnderlyingForLp = (lpAmount * totalUnderlying) / totalLp;

    expect(await ethYield.lpToUnderlying(lpAmount)).to.be.closeTo(expectedUnderlyingForLp, 1);

    const underlyingAmount = totalUnderlying / 4n;
    const expectedLp = (totalLp * underlyingAmount) / totalUnderlying;
    expect(await ethYield.underlyingToLp(underlyingAmount)).to.be.closeTo(expectedLp, 1);
  });
});

describe('EthYield onlyOwner actions', () => {
  it('authorizeUpgrade', async () => {
    const { owner, ethYield } = await loadFixture(createEthYieldFork);
    expect(function () {
      ethYield.interface.getFunctionName('upgradedTest');
    }).to.throw(TypeError);

    const ethYieldV2 = await upgrades.upgradeProxy(
      ethYield.target,
      await new EthYieldUpgradeTest__factory().connect(owner)
    );

    expect(await ethYieldV2.upgradedTest()).to.be.true;
  });

  it('authorizeUpgrade, not owner', async () => {
    const { owner, ethYield } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();
    expect(user.address).to.be.not.eq(owner.address);
    await expect(
      upgrades.upgradeProxy(ethYield.target, await new EthYieldUpgradeTest__factory().connect(user))
    ).to.be.revertedWithCustomError(ethYield, 'OwnableUnauthorizedAccount');
  });
});

describe('EthYield init errors', () => {
  it('wrong operator', async () => {
    const [owner, notOperator] = await ethers.getSigners();
    await expect(
      deployEthYieldContract(
        owner,
        EthAddressData.stEth,
        EthAddressData.weth,
        EthAddressData.elStrategy,
        EthAddressData.elStrategyManager,
        EthAddressData.elDelegationManager,
        notOperator.address
      )
    ).to.be.revertedWithCustomError({ interface: TestEthYield__factory.createInterface() }, 'WrongOperator');
  });

  it('wrong strategy', async () => {
    const [owner, notStrategy] = await ethers.getSigners();
    await expect(
      deployEthYieldContract(
        owner,
        EthAddressData.stEth,
        EthAddressData.weth,
        notStrategy.address,
        EthAddressData.elStrategyManager,
        EthAddressData.elDelegationManager,
        EthAddressData.eigenLayerOperator
      )
    ).to.be.revertedWithCustomError({ interface: TestEthYield__factory.createInterface() }, 'WrongStrategy');
  });

  it('wrong underlying token', async () => {
    const [owner] = await ethers.getSigners();
    await expect(
      deployEthYieldContract(
        owner,
        EthAddressData.weth,
        EthAddressData.weth,
        EthAddressData.elStrategy,
        EthAddressData.elStrategyManager,
        EthAddressData.elDelegationManager,
        EthAddressData.eigenLayerOperator
      )
    ).to.be.revertedWithCustomError({ interface: TestEthYield__factory.createInterface() }, 'UnknownToken');
  });

  it('weth zero address', async () => {
    const [owner] = await ethers.getSigners();
    await expect(
      deployEthYieldContract(
        owner,
        EthAddressData.stEth,
        ethers.ZeroAddress,
        EthAddressData.elStrategy,
        EthAddressData.elStrategyManager,
        EthAddressData.elDelegationManager,
        EthAddressData.eigenLayerOperator
      )
    ).to.be.revertedWithCustomError({ interface: TestEthYield__factory.createInterface() }, 'ZeroAddress');
  });
});

describe('EthYield errors', () => {
  it("Can't receive native eth from random address", async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();
    await expect(user.sendTransaction({ to: ethYield.target, value: parseEther('1') })).to.be.revertedWithCustomError(
      ethYield,
      'ReceiveValueFail'
    );
  });

  it('no element in eigenLayer queue', async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);
    await expect(ethYield.getEigenLayerWithdrawalQueueElement(0)).to.be.revertedWithCustomError(
      ethYield,
      'NoElementWithIndex'
    );
  });

  it('no element in lido queue', async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);
    await expect(ethYield.getLidoWithdrawalQueueElement(0)).to.be.revertedWithCustomError(
      ethYield,
      'NoElementWithIndex'
    );
  });

  it('stake can be called by axelar only', async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);
    await expect(ethYield.stake(1, parseEther('1'))).to.be.revertedWithoutReason();
  });

  it('unstake can be called by axelar only', async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);
    await expect(ethYield.unstake(1, parseEther('1'))).to.be.revertedWithoutReason();
  });

  it('reinit can be called by axelar only', async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);
    await expect(ethYield.reinit()).to.be.revertedWithoutReason();
  });
});
