import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';
import { ethers, upgrades } from 'hardhat';
import {
  MintableERC20__factory,
  TestAxelarGasService__factory,
  TestAxelarGateway__factory,
  TestWardenHandler__factory,
} from '../../typechain-types';
import { EVM_CHAIN_NAME } from './utils';

export const WardenChain = 'warden';
export const WardenContractAddress = 'warden-contract-1';
export const CommandId = '0x0000000000000000000000000000000000000000000000000000000000000000';

export enum ActionType {
  Stake = 0,
  Unstake = 1,
  Reinit = 2,
}

export enum Status {
  Success = 0,
  Failed = 1,
}

export async function testWardenHandlerFixture() {
  const [owner] = await ethers.getSigners();

  const testToken = await new MintableERC20__factory().connect(owner).deploy('test token', 'TT');
  const axelarGateway = await new TestAxelarGateway__factory().connect(owner).deploy();
  axelarGateway.addTokenAddress(await testToken.symbol(), await testToken.getAddress());

  const axelarGasService = await new TestAxelarGasService__factory().connect(owner).deploy();

  const wardenHandler = await new TestWardenHandler__factory().connect(owner).deploy();

  await wardenHandler.initialize(
    await axelarGateway.getAddress(),
    await axelarGasService.getAddress(),
    EVM_CHAIN_NAME,
    WardenChain,
    WardenContractAddress
  );

  return { axelarGateway, axelarGasService, wardenHandler, testToken, owner };
}
