import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import snapshotGasCost from '@uniswap/snapshot-gas-cost';
import { createAaveEthFork } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { USER_WARDEN_ADDRESS, setTokenBalance } from '../shared/utils';

describe('AaveYield gas', () => {
  it('user stake', async () => {
    const { aaveYield, weth9 } = await loadFixture(createAaveEthFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);
    await weth9.connect(user).approve(aaveYield.target, input);

    await snapshotGasCost(Number(await aaveYield.connect(user).stake.estimateGas(weth9, input, USER_WARDEN_ADDRESS)));
  });

  it('user unstake', async () => {
    const { aaveYield, weth9 } = await loadFixture(createAaveEthFork);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);
    await weth9.connect(user).approve(aaveYield.target, input);
    await aaveYield.connect(user).stake(weth9, input, USER_WARDEN_ADDRESS);

    await snapshotGasCost(Number(await aaveYield.connect(user).unstake.estimateGas(weth9, input)));
  });
});
