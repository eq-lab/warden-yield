import { expect } from 'chai';
import { ethers, upgrades } from 'hardhat';
import { EthAddressData } from '../shared/utils';
import { AaveYield__factory, EthYield__factory, Ownable2StepUpgradeable__factory } from '../../typechain-types';
import { HardhatEthersSigner } from '@nomicfoundation/hardhat-ethers/signers';
import { setBalance } from '@nomicfoundation/hardhat-network-helpers';
import { BaseContract, parseEther } from 'ethers';
import { WardenChain, WardenContractAddress } from '../shared/warden-handler-fixtures';

const TOTAL_SHARES_V1_ABI = `[{
    "inputs": [
      {
        "internalType": "address",
        "name": "token",
        "type": "address"
      }
    ],
    "name": "totalShares",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  }]`;

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

    const ethYieldV1 = new BaseContract(ethYield, TOTAL_SHARES_V1_ABI, ethers.provider);
    const totalSharesBefore = await ethYieldV1.connect(owner).getFunction('totalShares')(EthAddressData.weth);

    const lidoQueue = EthAddressData.lidoWithdrawalQueue;
    const axelarGateway = EthAddressData.axelarGateway;
    const axelarGasService = EthAddressData.axelarGasService;
    const wardenChain = WardenChain;
    const wardenContractAddress = WardenContractAddress;

    await upgrades.upgradeProxy(ethYield, await new EthYield__factory().connect(owner), {
      call: {
        fn: 'initializeV2',
        args: [lidoQueue, axelarGateway, axelarGasService, wardenChain, wardenContractAddress],
      },
    });

    const ethYieldV2 = await EthYield__factory.connect(ethYield, ethers.provider);

    expect(await ethYieldV2.totalShares()).to.be.eq(totalSharesBefore);
    // TODO add lpTokens check
  });

  it('AaveYield usdc upgrade', async () => {
    const aaveYieldUsdc = EthAddressData.aaveYieldUsdc;
    const owner = await getImpersonatedOwner(aaveYieldUsdc);

    const aaveYieldV1 = new BaseContract(aaveYieldUsdc, TOTAL_SHARES_V1_ABI, ethers.provider);
    const totalSharesBefore = await aaveYieldV1.connect(owner).getFunction('totalShares')(EthAddressData.usdc);

    const underlyingToken = EthAddressData.usdc;
    const axelarGateway = EthAddressData.axelarGateway;
    const axelarGasService = EthAddressData.axelarGasService;
    const wardenChain = WardenChain;
    const wardenContractAddress = WardenContractAddress;

    await upgrades.upgradeProxy(aaveYieldUsdc, await new AaveYield__factory().connect(owner), {
      call: {
        fn: 'initializeV2',
        args: [underlyingToken, axelarGateway, axelarGasService, wardenChain, wardenContractAddress],
      },
    });

    const aaveYieldV2 = await EthYield__factory.connect(aaveYieldUsdc, ethers.provider);
    expect(await aaveYieldV2.totalShares()).to.be.eq(totalSharesBefore);
    // TODO add lpTokens check
  });

  it('AaveYield usdt upgrade', async () => {
    const aaveYieldUsdt = EthAddressData.aaveYieldUsdt;
    const owner = await getImpersonatedOwner(aaveYieldUsdt);

    const aaveYieldV1 = new BaseContract(aaveYieldUsdt, TOTAL_SHARES_V1_ABI, ethers.provider);
    const totalSharesBefore = await aaveYieldV1.connect(owner).getFunction('totalShares')(EthAddressData.usdt);

    const underlyingToken = EthAddressData.usdt;
    const axelarGateway = EthAddressData.axelarGateway;
    const axelarGasService = EthAddressData.axelarGasService;
    const wardenChain = WardenChain;
    const wardenContractAddress = WardenContractAddress;

    await upgrades.upgradeProxy(aaveYieldUsdt, await new AaveYield__factory().connect(owner), {
      call: {
        fn: 'initializeV2',
        args: [underlyingToken, axelarGateway, axelarGasService, wardenChain, wardenContractAddress],
      },
    });

    const aaveYieldV2 = await EthYield__factory.connect(aaveYieldUsdt, ethers.provider);
    expect(await aaveYieldV2.totalShares()).to.be.eq(totalSharesBefore);
    // TODO add lpTokens check
  });
});