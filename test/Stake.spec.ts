import { expect } from 'chai';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers } from 'hardhat';
import { parseUnits } from 'ethers';
import { testFixture } from './shared/fixtures';

describe('Staking', () => {
  it('stake', async () => {
    const { owner, token, primary, yieldStorage } = await loadFixture(testFixture);

    const [_, user] = await ethers.getSigners();
    const stakeAmount = parseUnits('10', 18);
    await token.connect(user).approve(yieldStorage.target, stakeAmount);
    await yieldStorage.connect(user).stake(stakeAmount);

    expect(await yieldStorage.getInputAmount(user.address)).to.be.eq(stakeAmount);
    expect(await yieldStorage.getStakedAmount(user.address)).to.be.eq(stakeAmount);
    expect(await primary.totalAmount()).to.be.eq(stakeAmount);
  });
});
