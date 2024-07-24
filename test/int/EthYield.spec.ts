import { expect } from 'chai';
import { loadFixture, mine } from '@nomicfoundation/hardhat-network-helpers';
import { createEthYieldFork, deployEthYieldContract } from '../shared/fixtures';
import { ethers, upgrades } from 'hardhat';
import { parseEther } from 'ethers';
import {
  EthAddressData,
  WardenChain,
  WardenContractAddress,
  encodeStakeAction,
  encodeUnstakeAction,
  finalizeLidoWithdraw,
} from '../shared/utils';
import { EthYieldUpgradeTest__factory, TestEthYield__factory } from '../../typechain-types';
import { ActionType, CommandId } from '../shared/warden-handler-fixtures';

describe('EthYield stake', () => {
  it('user stake, weth', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, eigenLayerStrategy, ethYield, weth9, stEth } =
      await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const userEthBalanceBefore = await user.provider.getBalance(user.address);
    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    // here we simulate stake request coming from Axelar gateway
    const input = parseEther('1');
    await weth9.connect(user).deposit({ value: input });
    await weth9.connect(user).transfer(ethYield.target, input);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await ethYield.executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', input);

    const userEthBalanceAfter = await user.provider.getBalance(user.address);
    expect(userEthBalanceBefore - userEthBalanceAfter).to.be.gte(input);

    const contractShares = await eigenLayerStrategy.shares(ethYield.target);
    expect(contractShares).to.be.eq(await ethYield.totalShares());

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(ethYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);

    expect(await stEth.balanceOf(ethYield.target)).to.be.lessThanOrEqual(1);
  });

  it('user stake, zero amount', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, ethYield } = await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

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
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const stakeAmount = parseEther('1');
    await weth9.connect(user).deposit({ value: stakeAmount });
    await weth9.connect(user).transfer(ethYield.target, stakeAmount);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await ethYield.executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', stakeAmount);

    const elWithdrawFilter = ethYield.filters.EigenLayerWithdrawStart;
    const lidoWithdrawFilter = ethYield.filters.LidoWithdrawStart;

    const totalSharesBefore = await ethYield.totalShares();
    const sharesToUnstake = totalSharesBefore / 2n;

    const unstakeId = 1;
    const unstakePayload = encodeUnstakeAction(unstakeId, sharesToUnstake);
    await ethYield.execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);

    const [elWithdrawStartEvent] = await ethYield.queryFilter(elWithdrawFilter, -1);
    const elElement = await ethYield.getEigenLayerWithdrawalQueueElement(0);
    expect(elElement.unstakeId).to.be.eq(elWithdrawStartEvent.args[0]);
    expect(elElement.shares).to.be.eq(elWithdrawStartEvent.args[1]);
    expect(elElement.blockNumber).to.be.eq(elWithdrawStartEvent.blockNumber);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    await ethYield.connect(user).executeReinit();

    const [lidoWithdrawStartEvent] = await ethYield.queryFilter(lidoWithdrawFilter, -1);
    const lidoElement = await ethYield.getLidoWithdrawalQueueElement(0);
    expect(lidoElement.requestId).to.be.eq(await lidoWithdrawalQueue.getLastRequestId());
    expect(lidoElement.requested).to.be.eq(lidoWithdrawStartEvent.args[1]);

    const balanceBefore = await user.provider.getBalance(ethYield.target);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, lidoElement.requestId);
    await ethYield.connect(user).executeReinit();

    const balanceAfter = await user.provider.getBalance(ethYield.target);
    expect(balanceAfter).to.be.eq(balanceBefore + lidoElement.requested);

    expect(await ethYield.totalShares()).to.be.eq(totalSharesBefore - sharesToUnstake);
  });

  it('too low withdraw', async () => {
    const { ethYield, lidoWithdrawalQueue, eigenLayerStrategy, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const stakeAmount = parseEther('1');
    await weth9.connect(user).deposit({ value: stakeAmount });
    await weth9.connect(user).transfer(ethYield.target, stakeAmount);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    await ethYield
      .connect(user)
      .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', stakeAmount);

    //TODO: use lpAmount when implemented
    const minShares = await eigenLayerStrategy.underlyingToSharesView(
      await lidoWithdrawalQueue.MIN_STETH_WITHDRAWAL_AMOUNT()
    );
    const unstakeId = 1;
    const unstakePayload = encodeUnstakeAction(unstakeId, minShares);
    await ethYield.connect(user).execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);

    const [requestFailed] = await ethYield.queryFilter(ethYield.filters.RequestFailed, -1);
    expect(requestFailed.args[0]).to.be.eq(ActionType.Unstake);
    expect(requestFailed.args[1]).to.be.eq(unstakeId);
    // cast sig "LowWithdrawalAmount(uint256)" = 0x9d7ecf5d
    expect(requestFailed.args[2].startsWith('0x9d7ecf5d')).to.be.true;
    expect(requestFailed.args[2].endsWith(ethers.toBeHex(minShares).replace('0x', ''))).to.be.true;
  });

  it('lowest allowed unstake passes', async () => {
    const { ethYield, lidoWithdrawalQueue, eigenLayerStrategy, eigenLayerDelegationManager, weth9 } =
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

    const minAllowedShares =
      (await eigenLayerStrategy.underlyingToSharesView(await lidoWithdrawalQueue.MIN_STETH_WITHDRAWAL_AMOUNT())) + 1n;

    const unstakeId = 1;
    const unstakePayload = encodeUnstakeAction(unstakeId, minAllowedShares);
    await ethYield.connect(user).execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user).executeReinit();

    const lidoElement = await ethYield.getLidoWithdrawalQueueElement(0);
    const balanceBefore = await user.provider.getBalance(ethYield.target);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, lidoElement.requestId);
    await ethYield.connect(user).executeReinit();

    const balanceAfter = await user.provider.getBalance(ethYield.target);
    expect(balanceAfter).to.be.eq(balanceBefore + lidoElement.requested);
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
});
