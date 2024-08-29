import { loadFixture, mine } from '@nomicfoundation/hardhat-network-helpers';
import { expect } from 'chai';
import snapshotGasCost from '@uniswap/snapshot-gas-cost';
import { createEthYieldFork } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { ContractTransactionReceipt, parseEther } from 'ethers';
import {
  WardenChain,
  WardenContractAddress,
  encodeReinitAction,
  encodeStakeAction,
  encodeUnstakeAction,
  finalizeLidoWithdraw,
} from '../shared/utils';
import { CommandId } from '../shared/warden-handler-fixtures';
import { EthYield, IWETH9 } from '../../typechain-types';
import { HardhatEthersSigner } from '@nomicfoundation/hardhat-ethers/signers';

async function stake(
  ethYield: EthYield,
  weth9: IWETH9,
  user: HardhatEthersSigner,
  amount = parseEther('1'),
  stakeId: number = 1
): Promise<ContractTransactionReceipt> {
  await weth9.connect(user).deposit({ value: amount });
  await weth9.connect(user).transfer(ethYield.target, amount);
  const stakePayload = encodeStakeAction(stakeId);
  const txReceipt = await (
    await ethYield
      .connect(user)
      .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, 'WETH', amount)
  ).wait();

  const [reqFailed] = await ethYield.queryFilter(ethYield.filters.RequestFailed, -1);
  expect(reqFailed).to.be.undefined;

  return txReceipt!;
}

async function unstake(
  ethYield: EthYield,
  user: HardhatEthersSigner,
  lpAmount: bigint,
  unstakeId: number = 1
): Promise<ContractTransactionReceipt> {
  const unstakePayload = encodeUnstakeAction(unstakeId, lpAmount);
  const txResponse = await ethYield
    .connect(user)
    .execute(CommandId, WardenChain, WardenContractAddress, unstakePayload);
  const txReceipt = await txResponse.wait();

  const [reqFailed] = await ethYield.queryFilter(ethYield.filters.RequestFailed, -1);
  expect(reqFailed).to.be.undefined;

  return txReceipt!;
}

async function reinitFromAxelar(ethYield: EthYield): Promise<ContractTransactionReceipt> {
  const reinitPayload = encodeReinitAction();
  const txReceipt = await (await ethYield.execute(CommandId, WardenChain, WardenContractAddress, reinitPayload)).wait();
  return txReceipt!;
}

async function reinit(ethYield: EthYield): Promise<ContractTransactionReceipt> {
  const txResponse = await ethYield.executeReinit();
  const txReceipt = await txResponse.wait();

  const [reqFailed] = await ethYield.queryFilter(ethYield.filters.RequestFailed, -1);
  expect(reqFailed).to.be.undefined;

  return txReceipt!;
}

describe('EthYield gas', () => {
  it('stake', async () => {
    const { ethYield, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const txReceipt = await stake(ethYield, weth9, user);
    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('unstake', async () => {
    const { ethYield, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    await stake(ethYield, weth9, user);

    const lpAmount = await ethYield.totalLpTokens();
    const txReceipt = await unstake(ethYield, user, lpAmount);

    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('reinit from evm, complete eigenLayer withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    await stake(ethYield, weth9, user);

    const lpAmount = await ethYield.totalLpTokens();
    await unstake(ethYield, user, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const txReceipt = await reinit(ethYield);
    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('reinit from Axelar, complete eigenLayer withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    await stake(ethYield, weth9, user);

    const lpAmount = await ethYield.totalLpTokens();
    await unstake(ethYield, user, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const txReceipt = await reinitFromAxelar(ethYield);
    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('reinit, complete lido withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    await stake(ethYield, weth9, user);

    const lpAmount = await ethYield.totalLpTokens();
    await unstake(ethYield, user, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    await reinit(ethYield);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    const txReceipt = await reinit(ethYield);
    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('stake with reinit', async () => {
    const { ethYield, eigenLayerDelegationManager, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user, user2] = await ethers.getSigners();

    await stake(ethYield, weth9, user);

    const lpAmount = await ethYield.totalLpTokens();
    await unstake(ethYield, user, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const txReceipt = await stake(ethYield, weth9, user);

    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('stake, complete lido withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user, user2] = await ethers.getSigners();

    await stake(ethYield, weth9, user);

    const lpAmount = await ethYield.totalLpTokens();
    await unstake(ethYield, user, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await reinit(ethYield);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    const txReceipt = await stake(ethYield, weth9, user);
    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('stake native, complete one eigenLayer and one lido withdraws', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user1, user2, user3] = await ethers.getSigners();

    await stake(ethYield, weth9, user1);
    await stake(ethYield, weth9, user2);

    const lpAmount = (await ethYield.totalLpTokens()) / 2n;
    await unstake(ethYield, user1, lpAmount);

    await unstake(ethYield, user2, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await reinit(ethYield);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    const txReceipt = await stake(ethYield, weth9, user3);
    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('unstake, complete eigenLayer withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user1, user2] = await ethers.getSigners();

    await stake(ethYield, weth9, user1);
    await stake(ethYield, weth9, user2);

    const lpAmount = (await ethYield.totalLpTokens()) / 2n;
    await unstake(ethYield, user1, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);

    const txReceipt = await unstake(ethYield, user2, lpAmount);

    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('unstake, complete lido withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user1, user2] = await ethers.getSigners();

    await stake(ethYield, weth9, user1);
    await stake(ethYield, weth9, user2);

    const lpAmount = (await ethYield.totalLpTokens()) / 2n;
    await unstake(ethYield, user1, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await reinit(ethYield);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    const txReceipt = await unstake(ethYield, user2, lpAmount);

    await snapshotGasCost(Number(txReceipt.gasUsed));
  });

  it('unstake, complete one eigenLayer and one lido withdraws', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue, weth9 } = await loadFixture(createEthYieldFork);
    const [_, user1, user2, user3] = await ethers.getSigners();

    await stake(ethYield, weth9, user1);
    await stake(ethYield, weth9, user2);
    await stake(ethYield, weth9, user3);

    let lpAmount = (await ethYield.totalLpTokens()) / 3n;
    await unstake(ethYield, user1, lpAmount);

    await unstake(ethYield, user2, lpAmount);

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await reinit(ethYield);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, (await ethYield.getLidoWithdrawalQueueElement(0)).requestId);

    lpAmount = await ethYield.totalLpTokens();

    const txReceipt = await unstake(ethYield, user3, lpAmount);

    await snapshotGasCost(Number(txReceipt.gasUsed));
  });
});
