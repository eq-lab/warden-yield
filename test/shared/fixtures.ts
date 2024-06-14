import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';
import { ethers, upgrades } from 'hardhat';
import { MintableERC20, MintableERC20__factory, YieldTest, YieldTest__factory } from '../../typechain-types';
import { parseUnits } from 'ethers';

export async function deployToken(owner: SignerWithAddress): Promise<MintableERC20> {
  return new MintableERC20__factory().connect(owner).deploy('test token', 'TT');
}

export async function deployYieldContract(
  owner: SignerWithAddress,
  tokenAddress: string,
  protocolAddress: string
): Promise<YieldTest> {
  return upgrades.deployProxy(new YieldTest__factory().connect(owner), [tokenAddress, tokenAddress, protocolAddress], {
    initializer: 'initialize',
  }) as unknown as Promise<YieldTest>;
}

export async function testFixture(): Promise<{
  owner: SignerWithAddress;
  token: MintableERC20;
  yieldContract: YieldTest;
}> {
  const users = (await ethers.getSigners()).slice(0, 5);
  const owner = users[0];

  const token = await deployToken(owner);
  const amount = parseUnits('100', 18);

  await Promise.all(users.map((user) => token.connect(owner).mint(user.address, amount)));

  const yieldContract = await deployYieldContract(owner, await token.getAddress(), ethers.ZeroAddress);

  return {
    owner,
    token,
    yieldContract,
  };
}
