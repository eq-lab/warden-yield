import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import snapshotGasCost from '@uniswap/snapshot-gas-cost';
import { createAaveEthFork } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { parseEther, TransactionReceipt } from 'ethers';
import {
  USER_WARDEN_ADDRESS,
  WardenChain,
  WardenContractAddress,
  encodeStakeAction,
  encodeUnstakeAction,
  setTokenBalance,
} from '../shared/utils';
import { IERC20Metadata__factory } from '../../typechain-types';
import { CommandId } from '../shared/warden-handler-fixtures';

describe('AaveYield gas', () => {
  it('user stake', async () => {
    const { aaveYield, weth9 } = await loadFixture(createAaveEthFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);

    await weth9.connect(user).transfer(aaveYield.target, input);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    const tokenSymbol = await IERC20Metadata__factory.connect(await weth9.getAddress(), user).symbol();

    // stake
    const txReceipt = await (
      await aaveYield
        .connect(user)
        .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, tokenSymbol, input)
    ).wait();

    await snapshotGasCost(Number(txReceipt?.gasUsed));
  });

  it('user unstake', async () => {
    const { aaveYield, weth9 } = await loadFixture(createAaveEthFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);

    //stake
    await weth9.connect(user).transfer(aaveYield.target, input);
    const stakeId = 1;
    const stakePayload = encodeStakeAction(stakeId);
    const tokenSymbol = await IERC20Metadata__factory.connect(await weth9.getAddress(), user).symbol();

    await aaveYield
      .connect(user)
      .executeWithToken(CommandId, WardenChain, WardenContractAddress, stakePayload, tokenSymbol, input);

    //unstake
    const unstakeId = 1;
    const lpAmount = parseEther('0.5');
    const unstakePayload = encodeUnstakeAction(unstakeId, lpAmount);

    const txReceipt = await (
      await aaveYield.connect(user).execute(CommandId, WardenChain, WardenContractAddress, unstakePayload)
    ).wait();

    await snapshotGasCost(Number(txReceipt?.gasUsed));
  });
});
