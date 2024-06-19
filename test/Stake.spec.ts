import { expect } from 'chai';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers } from 'hardhat';
import { parseUnits } from 'ethers';
import { testYieldStorageFixture } from './shared/fixtures';

describe('Staking', () => {
  it('stake', async () => {
    const { testYieldStorage } = await loadFixture(testYieldStorageFixture);

    const [_, user] = await ethers.getSigners();
    const stakeAmount = parseUnits('10', 18);
    await testYieldStorage.connect(user).stake(stakeAmount);

    const stakedAmount = await testYieldStorage.getStakedAmount(stakeAmount);

    expect(await testYieldStorage.userInputAmount(user.address)).to.be.eq(stakeAmount);
    expect(await testYieldStorage.userStakedAmount(user.address)).to.be.eq(stakedAmount);
    expect(await testYieldStorage.totalInputAmount()).to.be.eq(stakeAmount);
    expect(await testYieldStorage.totalStakedAmount()).to.be.eq(stakedAmount);
  });
});
