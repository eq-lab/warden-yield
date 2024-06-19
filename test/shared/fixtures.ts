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
  IStrategyManager,
  IStrategy,
  IStrategyManager__factory,
  IStrategy__factory,
  IDelegationManager,
  IDelegationManager__factory,
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
  eigenLayerStrategyManager: IStrategyManager;
  eigenLayerStrategy: IStrategy;
  eigenLayerDelegationManager: IDelegationManager;
  eigenLayerOperator: string;
  lidoYield: LidoYield;
  owner: SignerWithAddress;
}

export async function createLidoFork(): Promise<LidoForkTestData> {
  const weth9Address = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2';
  const stEthAddress = '0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84';
  const elStrategyManager = '0x858646372CC42E1A627fcE94aa7A7033e7CF075A';
  const elStrategy = '0x93c4b944D05dfe6df7645A86cd2206016c51564D';
  const elDelegationManager = '0x39053D51B77DC0d36036Fc1fCc8Cb819df8Ef37A';
  const eigenLayerOperator = '0x71C6F7Ed8C2d4925d0bAf16f6A85BB1736D412eb';

  const [owner] = await ethers.getSigners();

  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;

  const lidoYield = (await upgrades.deployProxy(
    await new LidoYield__factory().connect(owner),
    [stEthAddress, weth9Address, elStrategy, elStrategyManager, elDelegationManager, eigenLayerOperator],
    {
      initializer: 'initialize',
      txOverrides: {
        maxFeePerGas: maxFeePerGas,
      },
    }
  )) as unknown as LidoYield;

  const weth9 = ERC20__factory.connect(weth9Address, owner);
  const stEth = ERC20__factory.connect(stEthAddress, owner);
  const eigenLayerStrategyManager = IStrategyManager__factory.connect(elStrategyManager, owner);
  const eigenLayerStrategy = IStrategy__factory.connect(elStrategy, owner);
  const eigenLayerDelegationManager = IDelegationManager__factory.connect(elDelegationManager, owner);

  return {
    weth9,
    stEth,
    lidoYield,
    eigenLayerStrategyManager,
    eigenLayerStrategy,
    eigenLayerDelegationManager,
    eigenLayerOperator,
    owner,
  };
}
