import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';
import { ethers, upgrades } from 'hardhat';
import {
  AaveYield,
  AaveYield__factory,
  EthYield,
  EthYield__factory,
  ERC20,
  ERC20__factory,
  IAToken,
  IAToken__factory,
  IPool__factory,
  MintableERC20,
  MintableERC20__factory,
  TestYieldStorage,
  TestYieldStorage__factory,
  IStrategyManager,
  IStrategy,
  IStrategyManager__factory,
  IStrategy__factory,
  IDelegationManager,
  IDelegationManager__factory,
  IPool,
} from '../../typechain-types';
import { parseUnits } from 'ethers';
import { EthAddressData } from './utils';

export async function deployToken(owner: SignerWithAddress): Promise<MintableERC20> {
  return new MintableERC20__factory().connect(owner).deploy('test token', 'TT');
}

export async function deployEthYieldContract(
  owner: SignerWithAddress,
  stEth: string,
  weth: string,
  elStrategy: string,
  elStrategyManager: string,
  elDelegationManager: string,
  eigenLayerOperator: string
): Promise<EthYield> {
  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;

  return upgrades.deployProxy(
    await new EthYield__factory().connect(owner),
    [stEth, weth, elStrategy, elStrategyManager, elDelegationManager, eigenLayerOperator],
    {
      initializer: 'initialize',
      txOverrides: {
        maxFeePerGas: maxFeePerGas,
      },
    }
  ) as unknown as EthYield;
}

export async function deployAaveYieldContract(
  owner: SignerWithAddress,
  aavePoolAddress: string,
  allowedTokens: string[]
): Promise<AaveYield> {
  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;
  return upgrades.deployProxy(await new AaveYield__factory().connect(owner), [aavePoolAddress, allowedTokens], {
    initializer: 'initialize',
    txOverrides: {
      maxFeePerGas: maxFeePerGas,
    },
  }) as unknown as AaveYield;
}

export async function deployTestYieldStorageContract(
  owner: SignerWithAddress,
  tokenAddress: string
): Promise<TestYieldStorage> {
  return upgrades.deployProxy(new TestYieldStorage__factory().connect(owner), [tokenAddress], {
    initializer: 'initialize',
  }) as unknown as Promise<TestYieldStorage>;
}

export async function testYieldStorageFixture(): Promise<{
  owner: SignerWithAddress;
  testYieldStorage: TestYieldStorage;
  weth9: MintableERC20;
}> {
  const users = (await ethers.getSigners()).slice(0, 5);
  const owner = users[0];

  const weth9 = await deployToken(owner);
  const amount = parseUnits('100', 18);

  await Promise.all(users.map((user) => weth9.connect(owner).mint(user.address, amount)));

  const testYieldStorage = await deployTestYieldStorageContract(owner, await weth9.getAddress());

  return {
    owner,
    testYieldStorage,
    weth9,
  };
}

export interface EthYieldForkTestData {
  weth9: ERC20;
  stEth: ERC20;
  eigenLayerStrategyManager: IStrategyManager;
  eigenLayerStrategy: IStrategy;
  eigenLayerDelegationManager: IDelegationManager;
  eigenLayerOperator: string;
  ethYield: EthYield;
  owner: SignerWithAddress;
}

export async function createEthYieldFork(): Promise<EthYieldForkTestData> {
  const [owner] = await ethers.getSigners();
  const ethYield = await deployEthYieldContract(
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
    ethYield,
    eigenLayerStrategyManager,
    eigenLayerStrategy,
    eigenLayerDelegationManager,
    eigenLayerOperator: EthAddressData.eigenLayerOperator,
    owner,
  };
}

export interface AaveForkTestData {
  weth9: ERC20;
  aEthWETH: IAToken;
  aaveYield: AaveYield;
  aavePool: IPool;
  owner: SignerWithAddress;
}

export async function createAaveEthFork(): Promise<AaveForkTestData> {
  const [owner] = await ethers.getSigners();
  const aaveYield = await deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [EthAddressData.weth]);

  const aavePool = IPool__factory.connect(EthAddressData.aaveEthPool, owner);
  const weth9 = ERC20__factory.connect(EthAddressData.weth, owner);
  const aEthWETH = IAToken__factory.connect(EthAddressData.aEth, owner);

  return {
    weth9,
    aEthWETH,
    aaveYield,
    aavePool,
    owner,
  };
}
