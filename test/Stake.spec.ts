import { expect } from 'chai';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers } from 'hardhat';
import { parseUnits } from 'ethers';
import { testYieldStorageFixture } from './shared/fixtures';
import { USER_WARDEN_ADDRESS } from './shared/utils';

describe('Staking', () => {
  it('stake', async () => {
    const { testYieldStorage, weth9 } = await loadFixture(testYieldStorageFixture);
    const weth9Address = await weth9.getAddress();
    const [_, user] = await ethers.getSigners();
    const stakeAmount = parseUnits('10', 18);
    await testYieldStorage.connect(user).stake(stakeAmount);

    const shares = await testYieldStorage.getStakedAmount(stakeAmount);

    expect(await testYieldStorage.totalShares()).to.be.eq(shares);
  });
});