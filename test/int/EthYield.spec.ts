import { expect } from 'chai';

import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { createEthYieldFork, deployEthYieldContract } from '../shared/fixtures';
import { ethers } from 'hardhat';
import { parseEther } from 'ethers';
import { EthAddressData, USER_WARDEN_ADDRESS, setTokenBalance } from '../shared/utils';
import { EthYield__factory } from '../../typechain-types';

describe('EthYield', () => {
  it('user stake, native', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, eigenLayerStrategy, EthYield, weth9 } =
      await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(EthYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const userEthBalanceBefore = await user.provider.getBalance(user.address);
    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    const input = parseEther('1');
    await EthYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input });

    expect(await EthYield.totalStakedAmount(weth9.target)).to.be.eq(input);
    expect(await EthYield.userStakedAmount(user.address, weth9.target)).to.be.eq(input);

    const userEthBalanceAfter = await user.provider.getBalance(user.address);
    expect(userEthBalanceBefore - userEthBalanceAfter).to.be.gte(input);

    const contractShares = await eigenLayerStrategy.shares(EthYield.target);
    expect(contractShares).to.be.eq(await EthYield.totalShares(weth9.target));

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(EthYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);
  });

  it('user stake, weth', async () => {
    const { eigenLayerStrategy, eigenLayerDelegationManager, eigenLayerOperator, weth9, EthYield } =
      await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(EthYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);
    const userWethBalanceBefore = await weth9.balanceOf(user.address);
    await weth9.connect(user).approve(EthYield.target, input);

    await EthYield.connect(user).stake(input, USER_WARDEN_ADDRESS);

    expect(await EthYield.totalStakedAmount(weth9.target)).to.be.eq(input);
    expect(await EthYield.userStakedAmount(user.address, weth9.target)).to.be.eq(input);

    const userWethBalanceAfter = await weth9.balanceOf(user.address);
    expect(userWethBalanceBefore - userWethBalanceAfter).to.be.eq(input);

    const contractShares = await eigenLayerStrategy.shares(EthYield.target);
    expect(contractShares).to.be.eq(await EthYield.totalShares(weth9.target));

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(EthYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);
  });

  it('user stake, wrong msg.value', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, EthYield } = await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(EthYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await expect(
      EthYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input - 1n })
    ).to.be.revertedWithCustomError(EthYield, 'WrongMsgValue');
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
    ).to.be.revertedWithCustomError({ interface: EthYield__factory.createInterface() }, 'WrongOperator');
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
    ).to.be.revertedWithCustomError({ interface: EthYield__factory.createInterface() }, 'UnknownToken');
  });
});
