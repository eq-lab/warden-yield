import { expect } from 'chai';

import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { createLidoFork, deployEthYieldContract } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { EthAddressData, setTokenBalance } from '../shared/utils';

describe('EthYield', () => {
  it('user stake, native', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, eigenLayerStrategy, EthYield } =
      await loadFixture(createLidoFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(EthYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const userEthBalanceBefore = await user.provider.getBalance(user.address);
    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    const input = parseEther('1');
    await EthYield.connect(user).stake(input, { value: input });

    expect(await EthYield.totalInputAmount()).to.be.eq(input);
    expect(await EthYield.userInputAmount(user.address)).to.be.eq(input);

    const userEthBalanceAfter = await user.provider.getBalance(user.address);
    expect(userEthBalanceBefore - userEthBalanceAfter).to.be.gte(input);

    const contractShares = await eigenLayerStrategy.shares(EthYield.target);
    expect(contractShares).to.be.eq(await EthYield.totalStakedAmount());

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(EthYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);
  });

  it('user stake, weth', async () => {
    const { eigenLayerStrategy, eigenLayerDelegationManager, eigenLayerOperator, weth9, EthYield } =
      await loadFixture(createLidoFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(EthYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);
    const userWethBalanceBefore = await weth9.balanceOf(user.address);
    await weth9.connect(user).approve(EthYield.target, input);

    await EthYield.connect(user).stake(input);

    expect(await EthYield.totalInputAmount()).to.be.eq(input);
    expect(await EthYield.userInputAmount(user.address)).to.be.eq(input);

    const userWethBalanceAfter = await weth9.balanceOf(user.address);
    expect(userWethBalanceBefore - userWethBalanceAfter).to.be.eq(input);

    const contractShares = await eigenLayerStrategy.shares(EthYield.target);
    expect(contractShares).to.be.eq(await EthYield.totalStakedAmount());

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(EthYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);
  });
});

describe('EthYield init errors', () => {
  it('wrong operator', async () => {
    const [owner, notOperator] = await ethers.getSigners();
    await expect(
      deployEthYieldContract(
        owner,
        EthAddressData.stEth,
        EthAddressData.weth,
        EthAddressData.elStrategy,
        EthAddressData.elStrategyManager,
        EthAddressData.elDelegationManager,
        notOperator.address
      )
    ).to.be.revertedWith('Wrong operator address');
  });

  it('wrong underlying token', async () => {
    const [owner] = await ethers.getSigners();
    await expect(
      deployEthYieldContract(
        owner,
        EthAddressData.weth,
        EthAddressData.weth,
        EthAddressData.elStrategy,
        EthAddressData.elStrategyManager,
        EthAddressData.elDelegationManager,
        EthAddressData.eigenLayerOperator
      )
    ).to.be.revertedWith('Wrong strategy or token');
  });
});
