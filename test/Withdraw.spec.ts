import { expect } from 'chai';
import { loadFixture, mine } from '@nomicfoundation/hardhat-network-helpers';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { testEigenLayerInteractorFixture, testLidoInteractorFixture } from './shared/fixtures';
import { finalizeLidoWithdraw } from './shared/utils';

describe('EigenLayer withdraw', () => {
  it('withdraw call', async () => {
    const { testEigenLayerInteractor, strategy } = await loadFixture(testEigenLayerInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const amount = parseEther('1');
    const txReceipt = await (await testEigenLayerInteractor.connect(user).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [event] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = event.args[0];
    const unstakeId = 1;
    await testEigenLayerInteractor.connect(user).withdraw(unstakeId, sharesToWithdraw);

    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(1);

    const queueElement = await testEigenLayerInteractor.getQueueElement(0);
    expect(queueElement.shares).to.be.eq(sharesToWithdraw);
    expect(queueElement.blockNumber).to.be.eq(txReceipt!.blockNumber + 1);
  });

  it('reinit withdraw completion', async () => {
    const { testEigenLayerInteractor, delegationManager, stEth } = await loadFixture(testEigenLayerInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const amount = parseEther('1');
    await (await testEigenLayerInteractor.connect(user).stake(amount, { value: amount })).wait();

    const stakeFilter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [stakeEvent] = await testEigenLayerInteractor.queryFilter(stakeFilter, -1);

    const sharesToWithdraw = stakeEvent.args[0];
    const unstakeId = 1;
    await testEigenLayerInteractor.connect(user).withdraw(unstakeId, sharesToWithdraw);

    const blocksToAwait = await delegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();

    await mine(blocksToAwait);
    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.reinit();

    const withdrawFilter = testEigenLayerInteractor.filters['EigenLayerWithdrawComplete(uint64,uint256)'];
    const [withdrawEvent] = await testEigenLayerInteractor.queryFilter(withdrawFilter, -1);
    const completeUnstakeId = withdrawEvent.args[0];
    const stEthWithdrawnAmount = withdrawEvent.args[1];

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.closeTo(stEthBalanceBefore + stEthWithdrawnAmount, 1);
    expect(completeUnstakeId).to.be.eq(unstakeId);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(0);
  });

  it('reinit too early', async () => {
    const { testEigenLayerInteractor, stEth, strategy } = await loadFixture(testEigenLayerInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const amount = parseEther('1');
    await (await testEigenLayerInteractor.connect(user).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [userEvent] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const unstakeId = 1;
    await testEigenLayerInteractor.connect(user).withdraw(unstakeId, userEvent.args[0]);

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
    const { testEigenLayerInteractor, delegationManager, stEth } = await loadFixture(testEigenLayerInteractorFixture);
    const [_, user1, user2] = await ethers.getSigners();

    const amount = parseEther('1');
    await (await testEigenLayerInteractor.connect(user1).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [userEvent] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = userEvent.args[0];
    const unstakeId = 1;
    await testEigenLayerInteractor.connect(user1).withdraw(unstakeId, sharesToWithdraw);

    const blocksToAwait = await delegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.connect(user2).stake(amount, { value: amount });

    const withdrawFilter = testEigenLayerInteractor.filters['EigenLayerWithdrawComplete(uint64,uint256)'];
    const [withdrawEvent] = await testEigenLayerInteractor.queryFilter(withdrawFilter, -1);
    const completeUnstakeId = withdrawEvent.args[0];
    const stEthWithdrawnAmount = withdrawEvent.args[1];

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.closeTo(stEthBalanceBefore + stEthWithdrawnAmount, 1);
    expect(completeUnstakeId).to.be.eq(unstakeId);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(0);
  });

  it('withdraw + reinit', async () => {
    const { testEigenLayerInteractor, delegationManager, stEth } = await loadFixture(testEigenLayerInteractorFixture);
    const [_, user1, user2] = await ethers.getSigners();

    const amount = parseEther('1');
    await testEigenLayerInteractor.connect(user1).stake(amount, { value: amount });
    await testEigenLayerInteractor.connect(user2).stake(amount, { value: amount });

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [user1Event, user2Event] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = user1Event.args[0];
    const unstakeId = 1;
    await testEigenLayerInteractor.connect(user1).withdraw(unstakeId, sharesToWithdraw);

    const blocksToAwait = await delegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.connect(user2).withdraw(unstakeId, user2Event.args[0]);

    const withdrawFilter = testEigenLayerInteractor.filters['EigenLayerWithdrawComplete(uint64,uint256)'];
    const [withdrawEvent] = await testEigenLayerInteractor.queryFilter(withdrawFilter, -1);
    const completeUnstakeId = withdrawEvent.args[0];
    const stEthWithdrawnAmount = withdrawEvent.args[1];

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.closeTo(stEthBalanceBefore + stEthWithdrawnAmount, 1);
    expect(completeUnstakeId).to.be.eq(unstakeId);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(2);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(1);
  });

  it('reinit 2+ withdraws in queue', async () => {
    const { testEigenLayerInteractor, delegationManager, stEth } = await loadFixture(testEigenLayerInteractorFixture);
    const [_, user1, user2] = await ethers.getSigners();

    const amount = parseEther('1');
    await (await testEigenLayerInteractor.connect(user1).stake(amount, { value: amount })).wait();
    await (await testEigenLayerInteractor.connect(user2).stake(amount, { value: amount })).wait();

    const filter = testEigenLayerInteractor.filters['Stake(uint256)'];
    const [user1Event, user2Event] = await testEigenLayerInteractor.queryFilter(filter, -1);

    const sharesToWithdraw = user1Event.args[0];
    const unstakeId = 1;
    const unstakeId2 = 2;
    await testEigenLayerInteractor.connect(user1).withdraw(unstakeId, sharesToWithdraw);
    await testEigenLayerInteractor.connect(user2).withdraw(unstakeId2, user2Event.args[0]);

    const blocksToAwait = await delegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const stEthBalanceBefore = await stEth.balanceOf(testEigenLayerInteractor.target);
    await testEigenLayerInteractor.reinit();

    const withdrawFilter = testEigenLayerInteractor.filters['EigenLayerWithdrawComplete(uint64,uint256)'];
    const [withdrawEvent] = await testEigenLayerInteractor.queryFilter(withdrawFilter, -1);
    const completeUnstakeId = withdrawEvent.args[0];
    const stEthWithdrawnAmount = withdrawEvent.args[1];

    const stEthBalanceAfter = await stEth.balanceOf(testEigenLayerInteractor.target);
    expect(stEthBalanceAfter).to.be.closeTo(stEthBalanceBefore + stEthWithdrawnAmount, 1);
    expect(completeUnstakeId).to.be.eq(unstakeId);
    expect(await testEigenLayerInteractor.getQueueStart()).to.be.eq(1);
    expect(await testEigenLayerInteractor.getQueueEnd()).to.be.eq(2);
    expect(await testEigenLayerInteractor.getQueueLength()).to.be.eq(1);

    const firstElement = await testEigenLayerInteractor.getQueueElement(0);
    expect(firstElement.shares).to.be.eq(user2Event.args[0]);
  });
});

describe('Lido withdraw', () => {
  it('withdraw call', async () => {
    const { testLidoInteractor } = await loadFixture(testLidoInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const amount = parseEther('1');
    await testLidoInteractor.connect(user).stake(amount, { value: amount });

    const sharesToWithdraw = amount;
    const unstakeId = 1;

    await testLidoInteractor.connect(user).withdraw(unstakeId, sharesToWithdraw);

    expect(await testLidoInteractor.getQueueLength()).to.be.eq(1);

    const queueElement = await testLidoInteractor.getQueueElement(0);
    expect(queueElement.requested).to.be.eq(sharesToWithdraw);
    expect(queueElement.requestId).to.be.not.eq(0);
  });

  it('reinit withdraw completion', async () => {
    const { testLidoInteractor, lidoWithdrawalQueue } = await loadFixture(testLidoInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const unstakeId = 1;
    const amount = parseEther('1');
    await (await testLidoInteractor.connect(user).stake(amount, { value: amount })).wait();

    await testLidoInteractor.connect(user).withdraw(unstakeId, amount);

    const queueElement = await testLidoInteractor.getQueueElement(0);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, queueElement.requestId);
    const wethalanceBefore = await user.provider.getBalance(testLidoInteractor.target);

    await testLidoInteractor.connect(user).reinit();

    const wethBalanceAfter = await user.provider.getBalance(testLidoInteractor.target);
    expect(wethBalanceAfter).to.be.eq(amount + wethalanceBefore);
    expect(await testLidoInteractor.getQueueStart()).to.be.eq(1);
    expect(await testLidoInteractor.getQueueEnd()).to.be.eq(1);
    expect(await testLidoInteractor.getQueueLength()).to.be.eq(0);
  });

  it('reinit too early', async () => {
    const { testLidoInteractor } = await loadFixture(testLidoInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const unstakeId = 1;
    const amount = parseEther('1');
    await (await testLidoInteractor.connect(user).stake(amount, { value: amount })).wait();

    await testLidoInteractor.connect(user).withdraw(unstakeId, amount);
    await testLidoInteractor.reinit();

    expect(await testLidoInteractor.getQueueStart()).to.be.eq(0);
    expect(await testLidoInteractor.getQueueEnd()).to.be.eq(1);
    expect(await testLidoInteractor.getQueueLength()).to.be.eq(1);

    const firstElement = await testLidoInteractor.getQueueElement(0);
    expect(firstElement.requested).to.be.eq(amount);
  });

  it('stake + reinit', async () => {
    const { testLidoInteractor, lidoWithdrawalQueue, weth } = await loadFixture(testLidoInteractorFixture);
    const [_, user1, user2] = await ethers.getSigners();

    const unstakeId = 1;
    const amount = parseEther('1');
    await (await testLidoInteractor.connect(user1).stake(amount, { value: amount })).wait();

    await testLidoInteractor.connect(user1).withdraw(unstakeId, amount);

    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await testLidoInteractor.getQueueElement(0)).requestId);

    const ethBalanceBefore = await user1.provider.getBalance(testLidoInteractor.target);
    await testLidoInteractor.connect(user2).stake(amount, { value: amount });

    const ethBalanceAfter = await user1.provider.getBalance(testLidoInteractor.target);
    expect(ethBalanceAfter).to.be.eq(ethBalanceBefore + amount);
    expect(await testLidoInteractor.getQueueStart()).to.be.eq(1);
    expect(await testLidoInteractor.getQueueEnd()).to.be.eq(1);
    expect(await testLidoInteractor.getQueueLength()).to.be.eq(0);
  });

  it('withdraw + reinit', async () => {
    const { testLidoInteractor, lidoWithdrawalQueue, weth } = await loadFixture(testLidoInteractorFixture);
    const [_, user1, user2] = await ethers.getSigners();

    const unstakeId = 1;
    const amount = parseEther('1');
    await testLidoInteractor.connect(user1).stake(amount, { value: amount });
    await testLidoInteractor.connect(user2).stake(amount, { value: amount });

    await testLidoInteractor.connect(user1).withdraw(unstakeId, amount);

    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await testLidoInteractor.getQueueElement(0)).requestId);

    const unstakeId2 = 2;
    const ethBalanceBefore = await user1.provider.getBalance(testLidoInteractor.target);
    await testLidoInteractor.connect(user2).withdraw(unstakeId2, amount);

    const ethBalanceAfter = await user1.provider.getBalance(testLidoInteractor.target);
    expect(ethBalanceAfter).to.be.eq(ethBalanceBefore + amount);
    expect(await testLidoInteractor.getQueueStart()).to.be.eq(1);
    expect(await testLidoInteractor.getQueueEnd()).to.be.eq(2);
    expect(await testLidoInteractor.getQueueLength()).to.be.eq(1);
  });

  it('reinit 2+ withdraws in queue', async () => {
    const { testLidoInteractor, lidoWithdrawalQueue, weth } = await loadFixture(testLidoInteractorFixture);
    const [_, user1, user2] = await ethers.getSigners();

    const unstakeId = 1;
    const unstakeId2 = 2;
    const amount = parseEther('1');
    await (await testLidoInteractor.connect(user1).stake(amount, { value: amount })).wait();
    await (await testLidoInteractor.connect(user2).stake(amount, { value: amount })).wait();

    await testLidoInteractor.connect(user1).withdraw(unstakeId, amount);
    await testLidoInteractor.connect(user2).withdraw(unstakeId2, amount);

    const secondWithdrawId = (await testLidoInteractor.getQueueElement(1)).requestId;

    await finalizeLidoWithdraw(lidoWithdrawalQueue, secondWithdrawId);

    const ethBalanceBefore = await user1.provider.getBalance(testLidoInteractor.target);
    await testLidoInteractor.reinit();

    const ethBalanceAfter = await user1.provider.getBalance(testLidoInteractor.target);
    expect(ethBalanceAfter).to.be.eq(ethBalanceBefore + amount);
    expect(await testLidoInteractor.getQueueStart()).to.be.eq(1);
    expect(await testLidoInteractor.getQueueEnd()).to.be.eq(2);
    expect(await testLidoInteractor.getQueueLength()).to.be.eq(1);

    const firstElement = await testLidoInteractor.getQueueElement(0);
    expect(firstElement.requestId).to.be.eq(secondWithdrawId);
  });

  it('big lido unstake split', async () => {
    const { testLidoInteractor, lidoWithdrawalQueue, weth } = await loadFixture(testLidoInteractorFixture);
    const [_, user] = await ethers.getSigners();

    const unstakeId = 1;
    const stakeAmount = parseEther('2010');
    const maxLidoWithdraw = await lidoWithdrawalQueue.MAX_STETH_WITHDRAWAL_AMOUNT();
    expect(stakeAmount).to.be.gt(maxLidoWithdraw);

    await testLidoInteractor.connect(user).stake(stakeAmount, { value: stakeAmount });
    await testLidoInteractor.connect(user).withdraw(unstakeId, stakeAmount);

    const withdrawsRequests = stakeAmount / maxLidoWithdraw;
    expect(withdrawsRequests).to.be.gte(1);
    for (let i = 0; i < withdrawsRequests; ++i) {
      const lidoElement = await testLidoInteractor.getQueueElement(i);
      expect(lidoElement.requested).to.be.eq(maxLidoWithdraw);
    }
    const lastLidoElement = await testLidoInteractor.getQueueElement(withdrawsRequests);

    const balanceBefore = await user.provider.getBalance(testLidoInteractor.target);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, lastLidoElement.requestId);
    await testLidoInteractor.reinit();

    const balanceAfter = await user.provider.getBalance(testLidoInteractor.target);
    expect(balanceAfter).to.be.eq(balanceBefore + maxLidoWithdraw);
  });
});
