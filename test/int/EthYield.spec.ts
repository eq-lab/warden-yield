import { expect } from 'chai';
import { loadFixture, mine } from '@nomicfoundation/hardhat-network-helpers';
import { createEthYieldFork, deployEthYieldContract } from '../shared/fixtures';
import { ethers, upgrades } from 'hardhat';
import { parseEther } from 'ethers';
import { EthAddressData, USER_WARDEN_ADDRESS, finalizeLidoWithdraw, setTokenBalance } from '../shared/utils';
import { EthYieldUpgradeTest__factory, EthYield__factory } from '../../typechain-types';

describe('EthYield stake', () => {
  it('user stake, native', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, eigenLayerStrategy, ethYield, weth9, stEth } =
      await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const userEthBalanceBefore = await user.provider.getBalance(user.address);
    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    const input = parseEther('1');
    await ethYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input });

    expect(await ethYield.totalStakedAmount(await ethYield.getWeth())).to.be.eq(input);
    expect(await ethYield.userStakedAmount(user.address, weth9.target)).to.be.eq(input);

    const userEthBalanceAfter = await user.provider.getBalance(user.address);
    expect(userEthBalanceBefore - userEthBalanceAfter).to.be.gte(input);

    const contractShares = await eigenLayerStrategy.shares(ethYield.target);
    expect(contractShares).to.be.eq(await ethYield.totalShares(weth9.target));

    expect(await ethYield.wardenAddress(user.address)).to.be.eq(USER_WARDEN_ADDRESS);

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(ethYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);

    expect(await stEth.balanceOf(ethYield.target)).to.be.lessThanOrEqual(1);
  });

  it('user stake, weth', async () => {
    const { eigenLayerStrategy, eigenLayerDelegationManager, eigenLayerOperator, weth9, ethYield, stEth } =
      await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const filter = eigenLayerDelegationManager.filters.OperatorSharesIncreased;

    const input = parseEther('1');
    await setTokenBalance(await weth9.getAddress(), user.address, input);
    const userWethBalanceBefore = await weth9.balanceOf(user.address);
    await weth9.connect(user).approve(ethYield.target, input);

    await ethYield.connect(user).stake(input, USER_WARDEN_ADDRESS);

    expect(await ethYield.totalStakedAmount(weth9.target)).to.be.eq(input);
    expect(await ethYield.userStakedAmount(user.address, weth9.target)).to.be.eq(input);

    const userWethBalanceAfter = await weth9.balanceOf(user.address);
    expect(userWethBalanceBefore - userWethBalanceAfter).to.be.eq(input);

    const contractShares = await eigenLayerStrategy.shares(ethYield.target);
    expect(contractShares).to.be.eq(await ethYield.totalShares(weth9.target));

    expect(await ethYield.wardenAddress(user.address)).to.be.eq(USER_WARDEN_ADDRESS);

    const [event] = await eigenLayerDelegationManager.queryFilter(filter, -1);
    expect(event.args[0]).to.be.eq(eigenLayerOperator);
    expect(event.args[1]).to.be.eq(ethYield.target);
    expect(event.args[2]).to.be.eq(eigenLayerStrategy.target);
    expect(event.args[3]).to.be.eq(contractShares);

    expect(await stEth.balanceOf(ethYield.target)).to.be.lessThanOrEqual(1);
  });

  it('user stake, wrong msg.value', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, ethYield } = await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    const input = parseEther('1');
    await expect(
      ethYield.connect(user).stake(input, USER_WARDEN_ADDRESS, { value: input - 1n })
    ).to.be.revertedWithCustomError(ethYield, 'WrongMsgValue');
  });

  it('user stake, zero amount', async () => {
    const { eigenLayerDelegationManager, eigenLayerOperator, ethYield } = await loadFixture(createEthYieldFork);
    // set up during EthYield contract init
    expect(await eigenLayerDelegationManager.delegatedTo(ethYield.target)).to.be.eq(eigenLayerOperator);
    const [_, user] = await ethers.getSigners();

    await expect(ethYield.connect(user).stake(0, USER_WARDEN_ADDRESS)).to.be.revertedWithCustomError(
      ethYield,
      'ZeroAmount'
    );
  });
});

describe('EthYield withdraw', () => {
  it('user withdraw', async () => {
    const { ethYield, eigenLayerDelegationManager, lidoWithdrawalQueue, eigenLayerStrategy } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();

    const stakeAmount = parseEther('1');
    await ethYield.connect(user).stake(stakeAmount, USER_WARDEN_ADDRESS, { value: stakeAmount });

    const userShares = await ethYield.userShares(user.address, await ethYield.getWeth());
    const txReceipt = await (await ethYield.connect(user).unstake(userShares)).wait();

    const elElement = await ethYield.getEigenLayerWithdrawalQueueElement(0);
    expect(elElement.shares).to.be.gt(0);
    // expect(elElement.underlyingAmount).to.be.eq(await eigenLayerStrategy.sharesToUnderlyingView(elElement.shares));
    expect(elElement.blockNumber).to.be.eq(txReceipt!.blockNumber)

    const blocksToAwait = await eigenLayerDelegationManager.MAX_WITHDRAWAL_DELAY_BLOCKS();
    await mine(blocksToAwait);
    await ethYield.connect(user).reinit();

    const lidoElement = await ethYield.getLidoWithdrawalQueueElement(0);
    expect(lidoElement.requestId).to.be.gt(0);
    expect(lidoElement.requested).to.be.gt(0);

    const balanceBefore = await user.provider.getBalance(ethYield.target);
    await finalizeLidoWithdraw(lidoWithdrawalQueue, lidoElement.requestId);
    await ethYield.reinit();

    const balanceAfter = await user.provider.getBalance(ethYield.target);
    expect(balanceAfter).to.be.eq(balanceBefore + lidoElement.requested);
  });
});

describe('EthYield onlyOwner actions', () => {
  it('authorizeUpgrade', async () => {
    const { owner, ethYield } = await loadFixture(createEthYieldFork);
    expect(function () {
      ethYield.interface.getFunctionName('upgradedTest');
    }).to.throw(TypeError);

    const ethYieldV2 = await upgrades.upgradeProxy(
      ethYield.target,
      await new EthYieldUpgradeTest__factory().connect(owner)
    );

    expect(await ethYieldV2.upgradedTest()).to.be.true;
  });

  it('authorizeUpgrade, not owner', async () => {
    const { owner, ethYield } = await loadFixture(createEthYieldFork);
    const [_, user] = await ethers.getSigners();
    expect(user.address).to.be.not.eq(owner.address);
    await expect(
      upgrades.upgradeProxy(ethYield.target, await new EthYieldUpgradeTest__factory().connect(user))
    ).to.be.revertedWithCustomError(ethYield, 'OwnableUnauthorizedAccount');
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

  it('wrong strategy', async () => {
    const [owner, notStrategy] = await ethers.getSigners();
    await expect(
      deployEthYieldContract(
        owner,
        EthAddressData.stEth,
        EthAddressData.weth,
        notStrategy.address,
        EthAddressData.elStrategyManager,
        EthAddressData.elDelegationManager,
        EthAddressData.eigenLayerOperator
      )
    ).to.be.revertedWithCustomError({ interface: EthYield__factory.createInterface() }, 'WrongStrategy');
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
