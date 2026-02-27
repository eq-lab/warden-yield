import { expect } from 'chai';
import { mine, setBalance } from '@nomicfoundation/hardhat-network-helpers';
import { ethers, upgrades } from 'hardhat';
import { parseEther } from 'ethers';
import {
  EthYield__factory,
  IDelegationManager__factory,
  Ownable2StepUpgradeable__factory,
  IERC20__factory,
  ILidoWithdrawalQueue__factory,
  IStrategy__factory,
} from '../../typechain-types';
import { HardhatEthersSigner } from '@nomicfoundation/hardhat-ethers/signers';

const ETH_YIELD_ADDRESS = '0x4DF66BCA96319C6A033cfd86c38BCDb9B3c11a72';
const LIDO_WITHDRAWAL_QUEUE = '0x889edc2edab5f40e902b864ad4d7ade8e412f9b1';
const DELEGATION_MANAGER = '0x39053D51B77DC0d36036Fc1fCc8Cb819df8Ef37A';
const STRATEGY = '0x93c4b944D05dfe6df7645A86cd2206016c51564D';
const WRAPPED_ETH = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2';
const USER = '0x32894d77fF28398eBa315FFA2775A79263866D4c';

async function getImpersonatedOwner(contractAddress: string): Promise<HardhatEthersSigner> {
  const ownable = await Ownable2StepUpgradeable__factory.connect(contractAddress, ethers.provider);
  const owner = await ethers.getImpersonatedSigner(await ownable.owner());
  setBalance(owner.address, parseEther('1'));
  return owner;
}

async function lidoFinalize(owner: HardhatEthersSigner) {
  const withdrawalQueue = ILidoWithdrawalQueue__factory.connect(LIDO_WITHDRAWAL_QUEUE, owner);
  const finalizerAddress = await withdrawalQueue.getRoleMember(await withdrawalQueue.FINALIZE_ROLE(), 0);
  const finalizer = await ethers.getImpersonatedSigner(finalizerAddress);

  const ethersToFinalize = await withdrawalQueue.unfinalizedStETH();
  await setBalance(finalizerAddress, 2n * ethersToFinalize);

  const lastRequestId = await withdrawalQueue.getLastRequestId();
  const maxShares = 10n ** 50n;

  await withdrawalQueue.connect(finalizer).finalize(lastRequestId, maxShares, { value: ethersToFinalize });
}

describe.only('EthYield', () => {
  it('EthYield withdrawals', async () => {
    const owner = await getImpersonatedOwner(ETH_YIELD_ADDRESS);

    const withdrawalQueue = ILidoWithdrawalQueue__factory.connect(LIDO_WITHDRAWAL_QUEUE, owner);
    const delegationManager = IDelegationManager__factory.connect(DELEGATION_MANAGER, owner);
    const strategy = IStrategy__factory.connect(STRATEGY, owner);
    const weth = IERC20__factory.connect(WRAPPED_ETH, owner);
    const user = await ethers.getImpersonatedSigner(USER);
    await setBalance(USER, 10n ** 18n);

    const shares = await strategy.shares(ETH_YIELD_ADDRESS);
    const underlying = await strategy.sharesToUnderlyingView(shares);

    await upgrades.upgradeProxy(ETH_YIELD_ADDRESS, new EthYield__factory().connect(owner), {
      call: {
        fn: 'initializeV2',
        args: [LIDO_WITHDRAWAL_QUEUE],
      },
    });

    const ethYield = EthYield__factory.connect(ETH_YIELD_ADDRESS, owner);

    expect(await ethYield.userWithdrawalsActive()).to.be.false;
    await expect(ethYield.connect(user).withdrawAll()).to.be.revertedWithCustomError(ethYield, 'Forbidden');

    let withdrawalData = await ethYield.getEigenLayerWithdrawalData();
    expect(withdrawalData[1]).to.be.not.eq(0);
    expect(withdrawalData[1]).to.be.eq(shares);

    const blocksToMine = await delegationManager.minWithdrawalDelayBlocks();
    await mine(blocksToMine);

    await ethYield.startLidoWithdrawal();

    expect(await ethYield.userWithdrawalsActive()).to.be.false;
    await expect(ethYield.connect(user).withdrawAll()).to.be.revertedWithCustomError(ethYield, 'Forbidden');

    withdrawalData = await ethYield.getEigenLayerWithdrawalData();
    expect(withdrawalData[1]).to.be.eq(0);
    expect(withdrawalData[1]).to.be.eq(0);

    expect(await ethYield.hasPendingLidoWithdrawal()).to.be.true;

    const requestId = await withdrawalQueue.getLastRequestId();
    let withdrawalStatus = (await withdrawalQueue.getWithdrawalStatus([requestId]))[0];

    expect(withdrawalStatus.owner).to.be.eq(ETH_YIELD_ADDRESS);
    expect(withdrawalStatus.amountOfStETH).to.be.closeTo(underlying, 100);
    expect(withdrawalStatus.isClaimed).to.be.false;
    expect(withdrawalStatus.isFinalized).to.be.false;

    await lidoFinalize(owner);
    withdrawalStatus = (await withdrawalQueue.getWithdrawalStatus([requestId]))[0];
    expect(withdrawalStatus.isFinalized).to.be.true;

    await ethYield.completeLidoWithdrawal();

    expect(await ethYield.userWithdrawalsActive()).to.be.true;

    withdrawalStatus = (await withdrawalQueue.getWithdrawalStatus([requestId]))[0];
    expect(withdrawalStatus.isClaimed).to.be.true;

    expect(await ethYield.hasPendingLidoWithdrawal()).to.be.false;
    expect(await weth.balanceOf(ETH_YIELD_ADDRESS)).to.be.eq(withdrawalStatus.amountOfStETH);

    const wethBalanceBefore = await weth.balanceOf(USER);

    const totalShares = await ethYield.totalShares(WRAPPED_ETH);
    const userShares = await ethYield.userShares(USER, WRAPPED_ETH);

    const withdrawalAmount = (withdrawalStatus.amountOfStETH * userShares) / totalShares;

    await ethYield.connect(user).withdrawAll();
    expect(await weth.balanceOf(USER)).to.be.eq(wethBalanceBefore + withdrawalAmount);
    expect(await weth.balanceOf(ETH_YIELD_ADDRESS)).to.be.eq(withdrawalStatus.amountOfStETH - withdrawalAmount);
    expect(await ethYield.userShares(USER, WRAPPED_ETH)).to.be.eq(0);
    expect(await ethYield.totalShares(WRAPPED_ETH)).to.be.eq(totalShares - userShares);

    await expect(ethYield.connect(user).withdrawAll()).to.be.revertedWithCustomError(ethYield, 'ZeroAmount');
  });
});
