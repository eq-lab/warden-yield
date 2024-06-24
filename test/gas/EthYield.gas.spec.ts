import { expect } from 'chai';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import snapshotGasCost from '@uniswap/snapshot-gas-cost';
import { createEthYieldFork } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { USER_WARDEN_ADDRESS, setTokenBalance } from '../shared/utils';

describe('EthYield', () => {
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
});
