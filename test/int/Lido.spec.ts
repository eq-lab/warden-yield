import { expect } from 'chai';

import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { createLidoFork } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { setTokenBalance } from '../shared/utils';

describe('LidoYield', () => {
  it('user stake, native', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, eigenLayerStrategy, lidoYield } =
      await loadFixture(createLidoFork);
    // set up during lidoYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(lidoYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const userEthBalanceBefore = await user.provider.getBalance(user.address);
    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    const input = parseEther('1');
    await lidoYield.connect(user).stake(input, { value: input });

    expect(await lidoYield.totalInputAmount()).to.be.eq(input);
    expect(await lidoYield.userInputAmount(user.address)).to.be.eq(input);

    const userEthBalanceAfter = await user.provider.getBalance(user.address);
    expect(userEthBalanceBefore - userEthBalanceAfter).to.be.gte(input);

    const contractShares = await eigenLayerStrategy.shares(lidoYield.target);
    expect(contractShares).to.be.eq(await lidoYield.totalStakedAmount());

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(lidoYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);
  });

  it('user stake, weth', async () => {
    const { eigenLayerStrategy, eigenLayerDelegationManager, eigenLayerOperator, weth9, lidoYield } =
      await loadFixture(createLidoFork);
    // set up during lidoYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(lidoYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);
    const userWethBalanceBefore = await weth9.balanceOf(user.address);
    await weth9.connect(user).approve(lidoYield.target, input);

    await lidoYield.connect(user).stake(input);

    expect(await lidoYield.totalInputAmount()).to.be.eq(input);
    expect(await lidoYield.userInputAmount(user.address)).to.be.eq(input);

    const userWethBalanceAfter = await weth9.balanceOf(user.address);
    expect(userWethBalanceBefore - userWethBalanceAfter).to.be.eq(input);

    const contractShares = await eigenLayerStrategy.shares(lidoYield.target);
    expect(contractShares).to.be.eq(await lidoYield.totalStakedAmount());

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(lidoYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);
  });
});
