import { expect } from 'chai';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers } from 'hardhat';
import { parseUnits } from 'ethers';
import { testFixture } from './shared/fixtures';

describe('Staking', () => {
  it('stake', async () => {
    const { owner, token, yieldContract } = await loadFixture(testFixture);

    const [_, user] = await ethers.getSigners();
    const stakeAmount = parseUnits('10', 18);
    await token.connect(user).approve(yieldContract.target, stakeAmount);
    await yieldContract.connect(user).stake(stakeAmount);

    expect(await yieldContract.userInputAmount(user.address)).to.be.eq(stakeAmount);
    expect(await yieldContract.userStakedAmount(user.address)).to.be.eq(stakeAmount);
    expect(await yieldContract.totalInputAmount()).to.be.eq(stakeAmount);
    expect(await yieldContract.totalStakedAmount()).to.be.eq(stakeAmount);
  });
});
