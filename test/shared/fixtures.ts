import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';
import { ethers, upgrades } from 'hardhat';
import {
  MintableERC20,
  MintableERC20__factory,
  LidoYield,
  LidoYield__factory,
  ERC20,
  ERC20__factory,
  TestYieldStorage,
  TestYieldStorage__factory,
} from '../../typechain-types';
import { parseUnits } from 'ethers';

export async function deployToken(owner: SignerWithAddress): Promise<MintableERC20> {
  return new MintableERC20__factory().connect(owner).deploy('test token', 'TT');
}

export async function deployYieldContract(
  owner: SignerWithAddress,
  tokenAddress: string,
  protocolAddress: string
): Promise<LidoYield> {
  return upgrades.deployProxy(new LidoYield__factory().connect(owner), [tokenAddress, tokenAddress, protocolAddress], {
    initializer: 'initialize',
  }) as unknown as Promise<LidoYield>;
}

export async function deployTestYieldStorageContract(
  owner: SignerWithAddress,
  tokenAddress: string,
  protocolAddress: string
): Promise<TestYieldStorage> {
  return upgrades.deployProxy(new TestYieldStorage__factory().connect(owner), [], {
    initializer: 'initialize',
  }) as unknown as Promise<TestYieldStorage>;
}

export async function testYieldStorageFixture(): Promise<{
  owner: SignerWithAddress;
  testYieldStorage: TestYieldStorage;
}> {
  const users = (await ethers.getSigners()).slice(0, 5);
  const owner = users[0];

  const token = await deployToken(owner);
  const amount = parseUnits('100', 18);

  await Promise.all(users.map((user) => token.connect(owner).mint(user.address, amount)));

  const testYieldStorage = await deployTestYieldStorageContract(owner, await token.getAddress(), ethers.ZeroAddress);

  return {
    owner,
    testYieldStorage,
  };
}

export interface LidoForkTestData {
  weth9: ERC20;
  stEth: ERC20;
  wStEth: ERC20;
  lidoYield: LidoYield;
  owner: SignerWithAddress;
}

export async function createLidoFork(): Promise<LidoForkTestData> {
  const weth9Address = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2';
  const stEthAddress = '0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84';
  const wStEthAddress = '0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0';

  const [owner] = await ethers.getSigners();

  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;

  const lidoYield = (await upgrades.deployProxy(
    await new LidoYield__factory().connect(owner),
    [stEthAddress, wStEthAddress, weth9Address],
    {
      initializer: 'initialize',
      txOverrides: {
        maxFeePerGas: maxFeePerGas,
      },
    }
  )) as unknown as LidoYield;

  const weth9 = ERC20__factory.connect(weth9Address, owner);
  const stEth = ERC20__factory.connect(stEthAddress, owner);
  const wStEth = ERC20__factory.connect(wStEthAddress, owner);

  return {
    weth9,
    stEth,
    wStEth,
    lidoYield,
    owner,
  };
}
