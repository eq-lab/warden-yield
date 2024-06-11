import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';
import { ethers, upgrades } from 'hardhat';
import {
  MintableERC20,
  MintableERC20__factory,
  YieldPrimaryTest,
  YieldPrimaryTest__factory,
  YieldStorage,
  YieldStorage__factory,
} from '../../typechain-types';
import { parseUnits } from 'ethers';

export async function deployToken(owner: SignerWithAddress): Promise<MintableERC20> {
  return new MintableERC20__factory().connect(owner).deploy('test token', 'TT');
}

export async function deployYieldStorage(
  owner: SignerWithAddress,
  primaryContract: string,
  token: string
): Promise<YieldStorage> {
  return upgrades.deployProxy(new YieldStorage__factory().connect(owner), [primaryContract, token], {
    initializer: 'initialize',
  }) as unknown as Promise<YieldStorage>;
}

export async function deployPrimaryContract(
  owner: SignerWithAddress,
  protocolAddress: string
): Promise<YieldPrimaryTest> {
  return upgrades.deployProxy(new YieldPrimaryTest__factory().connect(owner), [protocolAddress], {
    initializer: 'initialize',
  }) as unknown as Promise<YieldPrimaryTest>;
}

export async function testFixture(): Promise<{
  owner: SignerWithAddress;
  token: MintableERC20;
  primary: YieldPrimaryTest;
  yieldStorage: YieldStorage;
}> {
  const users = (await ethers.getSigners()).slice(0, 5);
  const owner = users[0];

  const token = await deployToken(owner);
  const amount = parseUnits('100', 18);

  await Promise.all(users.map(user => token.connect(owner).mint(user.address, amount)));

  const primary = await deployPrimaryContract(owner, ethers.ZeroAddress);
  const yieldStorage = await deployYieldStorage(owner, await primary.getAddress(), await token.getAddress());

  return {
    owner,
    token,
    primary,
    yieldStorage,
  };
}
