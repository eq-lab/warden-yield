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

    const shares = await testYieldStorage.getStakedAmount(stakeAmount);
    const lpt = await testYieldStorage.getLptAmount(shares, stakeAmount);

    await testYieldStorage.connect(user).stake(stakeAmount);

    expect(await testYieldStorage.totalShares()).to.be.eq(shares);
    expect(await testYieldStorage.totalLpTokens()).to.be.eq(lpt);
  });

  it('second stake', async () => {
    const { testYieldStorage } = await loadFixture(testYieldStorageFixture);
    const [_, user1, user2] = await ethers.getSigners();
    const stakeAmount = parseUnits('10', 18);

    await testYieldStorage.connect(user1).stake(stakeAmount);

    const sharesBefore = await testYieldStorage.totalShares();
    const lpBefore = await testYieldStorage.totalLpTokens();

    const shares = await testYieldStorage.getStakedAmount(stakeAmount);
    const lpt = await testYieldStorage.getLptAmount(shares, stakeAmount);

    await testYieldStorage.connect(user2).stake(stakeAmount);

    expect(await testYieldStorage.totalShares()).to.be.eq(shares + sharesBefore);
    expect(await testYieldStorage.totalLpTokens()).to.be.eq(lpt + lpBefore);
  });
});
