import { loadFixture, mine } from '@nomicfoundation/hardhat-network-helpers';
import snapshotGasCost from '@uniswap/snapshot-gas-cost';
import { createEthYieldFork } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { USER_WARDEN_ADDRESS, finalizeLidoWithdraw, setTokenBalance } from '../shared/utils';

describe('EthYield gas', () => {
  it('user stake, native', async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await snapshotGasCost(
      Number(await ethYield.connect(user).stake.estimateGas(input, USER_WARDEN_ADDRESS, { value: input }))
    );
  });

  it('user stake, weth', async () => {
    const { weth9, ethYield } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);
    await weth9.connect(user).approve(ethYield.target, input);

    await snapshotGasCost(Number(await ethYield.connect(user).stake.estimateGas(input, USER_WARDEN_ADDRESS)));
  });

  it('user unstake', async () => {
    const { ethYield } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user.address, await ethYield.getWeth());

    await snapshotGasCost(Number(await ethYield.connect(user).unstake.estimateGas(shares)));
  });

  it('reinit, complete eigenLayer withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user.address, await ethYield.getWeth());
    await ethYield.connect(user).unstake(shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    await snapshotGasCost(Number(await ethYield.connect(user).reinit.estimateGas()));
  });

  it('reinit, complete lido withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user.address, await ethYield.getWeth());
    await ethYield.connect(user).unstake(shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user).reinit();
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    await snapshotGasCost(Number(await ethYield.connect(user).reinit.estimateGas()));
  });

  it('stake native, complete eigenLayer withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager } = await loadFixture(createEthYieldFork);
    const [_, user, user2] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user.address, await ethYield.getWeth());
    await ethYield.connect(user).unstake(shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    await snapshotGasCost(
      Number(await ethYield.connect(user2).stake.estimateGas(input, USER_WARDEN_ADDRESS, { value: input }))
    );
  });

  it('stake native, complete lido withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue } = await loadFixture(createEthYieldFork);
    const [_, user, user2] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user.address, await ethYield.getWeth());
    await ethYield.connect(user).unstake(shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user).reinit();
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    await snapshotGasCost(
      Number(await ethYield.connect(user2).stake.estimateGas(input, USER_WARDEN_ADDRESS, { value: input }))
    );
  });

  it('stake native, complete one eigenLayer and one lido withdraws', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue } = await loadFixture(createEthYieldFork);
    const [_, user1, user2, user3] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user1).stake(input, USER_WARDEN_ADDRESS, { value: input });
    await ethYield.connect(user2).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const user1Shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(user1Shares);

    const user2Shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(user2Shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user1).reinit();
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    await snapshotGasCost(
      Number(await ethYield.connect(user3).stake.estimateGas(input, USER_WARDEN_ADDRESS, { value: input }))
    );
  });

  it('stake weth, complete eigenLayer withdraw', async () => {
    const { weth9, ethYield, eigenLayerDelegationManager } = await loadFixture(createEthYieldFork);
    const [_, user1, user2] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user1).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    await setTokenBalance(await weth9.getAddress(), user2.address, input);
    await weth9.connect(user2).approve(ethYield.target, input);

    await snapshotGasCost(Number(await ethYield.connect(user2).stake.estimateGas(input, USER_WARDEN_ADDRESS)));
  });

  it('stake weth, complete lido withdraw', async () => {
    const { weth9, ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue } = await loadFixture(createEthYieldFork);
    const [_, user1, user2] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user1).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user1).reinit();
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    await setTokenBalance(await weth9.getAddress(), user2.address, input);
    await weth9.connect(user2).approve(ethYield.target, input);

    await snapshotGasCost(Number(await ethYield.connect(user2).stake.estimateGas(input, USER_WARDEN_ADDRESS)));
  });

  it('stake weth, complete one eigenLayer and one lido withdraws', async () => {
    const { weth9, ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue } = await loadFixture(createEthYieldFork);
    const [_, user1, user2, user3] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user1).stake(input, USER_WARDEN_ADDRESS, { value: input });
    await ethYield.connect(user2).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const user1Shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(user1Shares);

    const user2Shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(user2Shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user1).reinit();
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    await setTokenBalance(await weth9.getAddress(), user3.address, input);
    await weth9.connect(user3).approve(ethYield.target, input);

    await snapshotGasCost(Number(await ethYield.connect(user3).stake.estimateGas(input, USER_WARDEN_ADDRESS)));
  });

  it('unstake, complete eigenLayer withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager } = await loadFixture(createEthYieldFork);
    const [_, user1, user2] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user1).stake(input, USER_WARDEN_ADDRESS, { value: input });
    await ethYield.connect(user2).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const user2Shares = await ethYield.userShares(user2.address, await ethYield.getWeth());

    await snapshotGasCost(Number(await ethYield.connect(user2).unstake.estimateGas(user2Shares)));
  });

  it('unstake, complete lido withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue } = await loadFixture(createEthYieldFork);
    const [_, user1, user2] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user1).stake(input, USER_WARDEN_ADDRESS, { value: input });
    await ethYield.connect(user2).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user1).reinit();
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    const user2Shares = await ethYield.userShares(user2.address, await ethYield.getWeth());

    await snapshotGasCost(Number(await ethYield.connect(user2).unstake.estimateGas(user2Shares)));
  });

  it('unstake, complete one eigenLayer and one lido withdraws', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue } = await loadFixture(createEthYieldFork);
    const [_, user1, user2, user3] = await ethers.getSigners();

    const input = parseEther('1');
    await ethYield.connect(user1).stake(input, USER_WARDEN_ADDRESS, { value: input });
    await ethYield.connect(user2).stake(input, USER_WARDEN_ADDRESS, { value: input });
    await ethYield.connect(user3).stake(input, USER_WARDEN_ADDRESS, { value: input });

    const user1Shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(user1Shares);

    const user2Shares = await ethYield.userShares(user1.address, await ethYield.getWeth());
    await ethYield.connect(user1).unstake(user2Shares);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user1).reinit();
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    const user3Shares = await ethYield.userShares(user3.address, await ethYield.getWeth());

    await snapshotGasCost(Number(await ethYield.connect(user3).unstake.estimateGas(user3Shares)));
  });
});
