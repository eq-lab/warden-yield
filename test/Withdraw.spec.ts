import { expect } from 'chai';
import { loadFixture, mine } from '@nomicfoundation/hardhat-network-helpers';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { testEigenLayerInteractorFixture } from './shared/fixtures';

describe('EigenLayer withdraw', () => {
  it('withdraw call', async () => {
    const { testEigenLayerInteractor } = await loadFixture(testEigenLayerInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const amount = parseEther('1');
    const txReceipt = await (await testEigenLayerInteractor.connect(user).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [event] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = event.args[0];
    await testEigenLayerInteractor.connect(user).withdraw(sharesToWithdraw);

    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(1);

    const queueElement = await testEigenLayerInteractor.getQueueElement(0);
    expect(queueElement.shares).to.be.eq(sharesToWithdraw);
    expect(queueElement.blockNumber).to.be.eq(txReceipt!.blockNumber + 1);
  });

  it('reinit withdraw completion', async () => {
    const { testEigenLayerInteractor, delegationManager, strategy, stEth } = await loadFixture(
      testEigenLayerInteractorFixture
    );
    const [_, user] = await ethers.getSigners();

    const amount = parseEther('1');
    await (await testEigenLayerInteractor.connect(user).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [event] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = event.args[0];
    await testEigenLayerInteractor.connect(user).withdraw(sharesToWithdraw);
    const stEthWithdrawnAmount = await strategy.sharesToUnderlyingView(sharesToWithdraw);

    const blocksToAwait = await delegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();

    await mine(blocksToAwait);
    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.reinit();

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.closeTo(stEthBalanceBefore + stEthWithdrawnAmount, 1);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(0);
  });

  it('reinit too early', async () => {
    const { testEigenLayerInteractor, stEth } = await loadFixture(testEigenLayerInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const amount = parseEther('1');
    await (await testEigenLayerInteractor.connect(user).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [userEvent] = await testEigenLayerInteractor.queryFilter(filter, -1);

    await testEigenLayerInteractor.connect(user).withdraw(userEvent.args[0]);

    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.reinit();

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.eq(stEthBalanceBefore);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(0);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(1);

    const firstElement = await testEigenLayerInteractor.getQueueElement(0);
    expect(firstElement.shares).to.be.eq(userEvent.args[0]);
  });

  it('stake + reinit', async () => {
    const { testEigenLayerInteractor, delegationManager, strategy, stEth } = await loadFixture(
      testEigenLayerInteractorFixture
    );
    const [_, user1, user2] = await ethers.getSigners();

    const amount = parseEther('1');
    await (await testEigenLayerInteractor.connect(user1).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [userEvent] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = userEvent.args[0];
    await testEigenLayerInteractor.connect(user1).withdraw(sharesToWithdraw);
    const stEthWithdrawnAmount = await strategy.sharesToUnderlyingView(sharesToWithdraw);

    const blocksToAwait = await delegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.connect(user2).stake(amount, { value: amount });

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.closeTo(stEthBalanceBefore + stEthWithdrawnAmount, 1);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(0);
  });

  it('withdraw + reinit', async () => {
    const { testEigenLayerInteractor, delegationManager, strategy, stEth } = await loadFixture(
      testEigenLayerInteractorFixture
    );
    const [_, user1, user2] = await ethers.getSigners();

    const amount = parseEther('1');
    await testEigenLayerInteractor.connect(user1).stake(amount, { value: amount });
    await testEigenLayerInteractor.connect(user2).stake(amount, { value: amount });

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [user1Event, user2Event] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = user1Event.args[0];
    await testEigenLayerInteractor.connect(user1).withdraw(sharesToWithdraw);
    const stEthWithdrawnAmount = await strategy.sharesToUnderlyingView(sharesToWithdraw);

    const blocksToAwait = await delegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.connect(user2).withdraw(user2Event.args[0]);

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.closeTo(stEthBalanceBefore + stEthWithdrawnAmount, 1);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(2);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(1);
  });

  it('reinit 2+ withdraws in queue', async () => {
    const { testEigenLayerInteractor, delegationManager, strategy, stEth } = await loadFixture(
      testEigenLayerInteractorFixture
    );
    const [_, user1, user2] = await ethers.getSigners();

    const amount = parseEther('1');
    await (await testEigenLayerInteractor.connect(user1).stake(amount, { value: amount })).wait();
    await (await testEigenLayerInteractor.connect(user2).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [user1Event, user2Event] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = user1Event.args[0];
    await testEigenLayerInteractor.connect(user1).withdraw(sharesToWithdraw);
    const stEthWithdrawnAmount = await strategy.sharesToUnderlyingView(sharesToWithdraw);

    await testEigenLayerInteractor.connect(user2).withdraw(user2Event.args[0]);

    const blocksToAwait = await delegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.reinit();

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.closeTo(stEthBalanceBefore + stEthWithdrawnAmount, 1);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(2);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(1);

    const firstElement = await testEigenLayerInteractor.getQueueElement(0);
    expect(firstElement.shares).to.be.eq(user2Event.args[0]);
  });
});
