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
import { EthAddressData } from './utils';

export async function deployToken(owner: SignerWithAddress): Promise<MintableERC20> {
  return new MintableERC20__factory().connect(owner).deploy('test token', 'TT');
}

export async function deployLidoYieldContract(
  owner: SignerWithAddress,
  stEth: string,
  weth: string,
  elStrategy: string,
  elStrategyManager: string,
  elDelegationManager: string,
  eigenLayerOperator: string
): Promise<LidoYield> {
  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;

  return upgrades.deployProxy(
    await new LidoYield__factory().connect(owner),
    [stEth, weth, elStrategy, elStrategyManager, elDelegationManager, eigenLayerOperator],
    {
      initializer: 'initialize',
      txOverrides: {
        maxFeePerGas: maxFeePerGas,
      },
    }
  ) as unknown as LidoYield;
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
  const [owner] = await ethers.getSigners();
  const lidoYield = await deployLidoYieldContract(
    owner,
    EthAddressData.stEth,
    EthAddressData.weth,
    EthAddressData.elStrategy,
    EthAddressData.elStrategyManager,
    EthAddressData.elDelegationManager,
    EthAddressData.eigenLayerOperator
  );
  const weth9 = ERC20__factory.connect(EthAddressData.weth, owner);
  const stEth = ERC20__factory.connect(EthAddressData.stEth, owner);
  const eigenLayerStrategyManager = IStrategyManager__factory.connect(EthAddressData.elStrategyManager, owner);
  const eigenLayerStrategy = IStrategy__factory.connect(EthAddressData.elStrategy, owner);
  const eigenLayerDelegationManager = IDelegationManager__factory.connect(EthAddressData.elDelegationManager, owner);

  return {
    weth9,
    stEth,
    lidoYield,
    eigenLayerStrategyManager,
    eigenLayerStrategy,
    eigenLayerDelegationManager,
    eigenLayerOperator: EthAddressData.eigenLayerOperator,
    owner,
  };
}
