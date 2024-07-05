import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';
import { ethers, upgrades } from 'hardhat';
import {
  ERC20,
  ERC20__factory,
  EthYield,
  EthYield__factory,
  IAToken,
  IAToken__factory,
  IDelegationManager,
  IDelegationManager__factory,
  IPool,
  IPool__factory,
  IStrategy,
  IStrategy__factory,
  IStrategyManager,
  IStrategyManager__factory,
  MintableERC20,
  MintableERC20__factory,
  TestAaveYield,
  TestAaveYield__factory,
  TestYieldStorage,
  TestYieldStorage__factory,
  TestEigenLayerInteractor__factory,
  TestEigenLayerInteractor,
  TestLidoInteractor,
  TestLidoInteractor__factory,
  ILidoWithdrawalQueueExtended,
  ILidoWithdrawalQueueExtended__factory,
} from '../../typechain-types';
import { parseUnits } from 'ethers';
import { EthAddressData } from './utils';

export async function deployToken(owner: SignerWithAddress): Promise<MintableERC20> {
  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;
  return new MintableERC20__factory().connect(owner).deploy('test token', 'TT', {
    maxFeePerGas: maxFeePerGas,
  });
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
): Promise<TestAaveYield> {
  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;
  return upgrades.deployProxy(await new TestAaveYield__factory().connect(owner), [aavePoolAddress, allowedTokens], {
    initializer: 'initialize',
    txOverrides: {
      maxFeePerGas: maxFeePerGas,
    },
  }) as unknown as TestAaveYield;
}

export async function deployTestYieldStorageContract(
  owner: SignerWithAddress,
  tokenAddress: string
): Promise<TestYieldStorage> {
  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;
  return upgrades.deployProxy(new TestYieldStorage__factory().connect(owner), [tokenAddress], {
    initializer: 'initialize',
    txOverrides: {
      maxFeePerGas: maxFeePerGas,
    },
  }) as unknown as Promise<TestYieldStorage>;
}

export async function deployTestLidoInteractor(
  owner: SignerWithAddress,
  weth: string,
  stEth: string,
  lidoWithdrawalQueue: string,
): Promise<TestLidoInteractor> {
  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;
  return upgrades.deployProxy(
    new TestLidoInteractor__factory().connect(owner),
    [weth, stEth, lidoWithdrawalQueue],
    {
      initializer: 'initialize',
      txOverrides: {
        maxFeePerGas: maxFeePerGas,
      },
    }
  ) as unknown as Promise<TestLidoInteractor>;
}

export async function deployTestEigenLayerInteractor(
  owner: SignerWithAddress,
  weth: string,
  stEth: string,
  elStrategy: string,
  elStrategyManager: string,
  elDelegationManager: string,
  eigenLayerOperator: string
): Promise<TestEigenLayerInteractor> {
  const blockNumber = await owner.provider.getBlockNumber();
  const maxFeePerGas = (await owner.provider.getBlock(blockNumber))!.baseFeePerGas! * 10n;
  return upgrades.deployProxy(
    new TestEigenLayerInteractor__factory().connect(owner),
    [weth, stEth, elStrategy, elStrategyManager, elDelegationManager, eigenLayerOperator],
    {
      initializer: 'initialize',
      txOverrides: {
        maxFeePerGas: maxFeePerGas,
      },
    }
  ) as unknown as Promise<TestEigenLayerInteractor>;
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

export async function testLidoInteractorFixture(): Promise<{
  owner: SignerWithAddress;
  testLidoInteractor: TestLidoInteractor;
  lidoWithdrawalQueue: ILidoWithdrawalQueueExtended;
  stEth: ERC20;
}> {
  const [owner] = await ethers.getSigners();

  const testLidoInteractor = await deployTestLidoInteractor(
    owner,
    EthAddressData.weth,
    EthAddressData.stEth,
    EthAddressData.lidoWithdrawalQueue,
  );

  const stEth = ERC20__factory.connect(EthAddressData.stEth, owner);
  const lidoWithdrawalQueue = ILidoWithdrawalQueueExtended__factory.connect(EthAddressData.lidoWithdrawalQueue, owner);

  return {
    owner,
    testLidoInteractor,
    lidoWithdrawalQueue,
    stEth,
  };
}

export async function testEigenLayerInteractorFixture(): Promise<{
  owner: SignerWithAddress;
  testEigenLayerInteractor: TestEigenLayerInteractor;
  delegationManager: IDelegationManager;
  strategy: IStrategy;
  stEth: ERC20;
}> {
  const [owner] = await ethers.getSigners();

  const testEigenLayerInteractor = await deployTestEigenLayerInteractor(
    owner,
    EthAddressData.weth,
    EthAddressData.stEth,
    EthAddressData.elStrategy,
    EthAddressData.elStrategyManager,
    EthAddressData.elDelegationManager,
    EthAddressData.eigenLayerOperator
  );

  const delegationManager = IDelegationManager__factory.connect(EthAddressData.elDelegationManager, owner);
  const stEth = ERC20__factory.connect(EthAddressData.stEth, owner);
  const strategy = IStrategy__factory.connect(EthAddressData.elStrategy, owner);

  return {
    owner,
    testEigenLayerInteractor,
    delegationManager,
    strategy,
    stEth,
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
  usdt: ERC20;
  aEthUsdt: IAToken;
  usdc: ERC20;
  aEthUsdc: IAToken;
  aaveYield: TestAaveYield;
  aavePool: IPool;
  owner: SignerWithAddress;
}

export async function createAaveEthFork(): Promise<AaveForkTestData> {
  const [owner] = await ethers.getSigners();
  const aaveYield = await deployAaveYieldContract(owner, EthAddressData.aaveEthPool, [
    EthAddressData.weth,
    EthAddressData.usdt,
    EthAddressData.usdc,
  ]);

  const aavePool = IPool__factory.connect(EthAddressData.aaveEthPool, owner);
  const weth9 = ERC20__factory.connect(EthAddressData.weth, owner);
  const aEthWETH = IAToken__factory.connect(EthAddressData.aEth, owner);
  const usdt = ERC20__factory.connect(EthAddressData.usdt, owner);
  const aEthUsdt = IAToken__factory.connect(EthAddressData.aEthUsdt, owner);
  const usdc = ERC20__factory.connect(EthAddressData.usdc, owner);
  const aEthUsdc = IAToken__factory.connect(EthAddressData.aEthUsdc, owner);

  return {
    weth9,
    aEthWETH,
    usdt,
    aEthUsdt,
    usdc,
    aEthUsdc,
    aaveYield,
    aavePool,
    owner,
  };
}
