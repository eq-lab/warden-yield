import { expect } from 'chai';

import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { createLidoFork } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { setTokenBalance } from '../shared/utils';

describe('LidoYield', () => {
  it('user stake, native', async () => {
    const { wStEth, lidoYield } = await loadFixture(createLidoFork);
    const [_, user] = await ethers.getSigners();

    const wStEthBalanceBefore = await wStEth.balanceOf(lidoYield.getAddress());
    const userEthBalanceBefore = await user.provider.getBalance(user.address);

    const input = parseEther('1');
    await lidoYield.connect(user).stake(input, { value: input });
    expect(await lidoYield.totalInputAmount()).to.be.eq(input);
    expect(await lidoYield.userInputAmount(user.address)).to.be.eq(input);

    const wStEthBalanceAfter = await wStEth.balanceOf(lidoYield.getAddress());
    const wStEthBalanceDelta = wStEthBalanceAfter - wStEthBalanceBefore;
    const userEthBalanceAfter = await user.provider.getBalance(user.address);

    expect(wStEthBalanceDelta).to.be.greaterThan(0);
    expect(await lidoYield.totalStakedAmount()).to.be.eq(wStEthBalanceDelta);
    expect(await lidoYield.userStakedAmount(user.address)).to.be.eq(wStEthBalanceDelta);
    expect(userEthBalanceBefore - userEthBalanceAfter).to.be.gte(input);
  });

  it('user stake, weth', async () => {
    const { wStEth, weth9, lidoYield } = await loadFixture(createLidoFork);
    const [_, user] = await ethers.getSigners();

    const wStEthBalanceBefore = await wStEth.balanceOf(lidoYield.getAddress());
    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);
    const userWethBalanceBefore = await weth9.balanceOf(user.address);
    await weth9.connect(user).approve(lidoYield.target, input);

    await lidoYield.connect(user).stake(input);
    expect(await lidoYield.totalInputAmount()).to.be.eq(input);
    expect(await lidoYield.userInputAmount(user.address)).to.be.eq(input);

    const wStEthBalanceAfter = await wStEth.balanceOf(lidoYield.getAddress());
    const wStEthBalanceDelta = wStEthBalanceAfter - wStEthBalanceBefore;
    const userWethBalanceAfter = await weth9.balanceOf(user.address);

    expect(wStEthBalanceDelta).to.be.greaterThan(0);
    expect(await lidoYield.totalStakedAmount()).to.be.eq(wStEthBalanceDelta);
    expect(await lidoYield.userStakedAmount(user.address)).to.be.eq(wStEthBalanceDelta);
    expect(userWethBalanceBefore - userWethBalanceAfter).to.be.eq(input);
  });
});
