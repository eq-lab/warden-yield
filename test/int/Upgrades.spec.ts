import { ethers, upgrades } from 'hardhat';
import { EthAddressData } from '../shared/utils';
import { AaveYield__factory, EthYield__factory, Ownable2StepUpgradeable__factory } from '../../typechain-types';
import { HardhatEthersSigner } from '@nomicfoundation/hardhat-ethers/signers';
import { setBalance } from '@nomicfoundation/hardhat-network-helpers';
import { parseEther } from 'ethers';

async function getImpersonatedOwner(contractAddress: string): Promise<HardhatEthersSigner> {
  const ownable = await Ownable2StepUpgradeable__factory.connect(contractAddress, ethers.provider);
  const owner = await ethers.getImpersonatedSigner(await ownable.owner());
  setBalance(owner.address, parseEther('1'));
  return owner;
}

describe('Upgrades', () => {
  it('EthYield upgrade', async () => {
    const ethYield = EthAddressData.ethYield;
    const owner = await getImpersonatedOwner(ethYield);
  
    await upgrades.upgradeProxy(ethYield, await new EthYield__factory().connect(owner));
  });

  it('AaveYield usdc upgrade', async () => {
    const aaveYield = EthAddressData.aaveYieldUsdc;
    const owner = await getImpersonatedOwner(aaveYield);
  
    await upgrades.upgradeProxy(aaveYield, await new AaveYield__factory().connect(owner));
  });

  it('AaveYield usdt upgrade', async () => {
    const aaveYield = EthAddressData.aaveYieldUsdt;
    const owner = await getImpersonatedOwner(aaveYield);
  
    await upgrades.upgradeProxy(aaveYield, await new AaveYield__factory().connect(owner));
  });
});
