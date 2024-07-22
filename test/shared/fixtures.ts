import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';
import { ethers, upgrades } from 'hardhat';
import {
  AaveYield__factory,
  AaveYield,
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
  TestYieldStorage,
  TestYieldStorage__factory,
  TestEigenLayerInteractor__factory,
  TestEigenLayerInteractor,
  TestLidoInteractor,
  TestLidoInteractor__factory,
  ILidoWithdrawalQueueExtended,
  ILidoWithdrawalQueueExtended__factory,
  TestWETH9,
  TestWETH9__factory,
  TestEthYield__factory,
  IWETH9__factory,
  IWETH9,
  TestAxelarGateway,
  TestAxelarGateway__factory,
} from '../../typechain-types';
import { parseUnits } from 'ethers';
import { EthAddressData, WardenChain, WardenContractAddress } from './utils';

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
  return upgrades.deployProxy(
    await new TestEthYield__factory().connect(owner),
    [stEth, weth, elStrategy, elStrategyManager, elDelegationManager, eigenLayerOperator],
    {
      initializer: 'initialize',
    }
  ) as unknown as EthYield;
}

export async function deployTestAxelarGateway(owner: SignerWithAddress): Promise<TestAxelarGateway> {
  const axelarGateway = await new TestAxelarGateway__factory().connect(owner).deploy();
  await axelarGateway.connect(owner).addTokenAddress('WETH', EthAddressData.weth);
  return axelarGateway;
}

export async function deployAaveYieldContract(
  owner: SignerWithAddress,
  aavePoolAddress: string,
  allowedTokens: string[]
): Promise<AaveYield> {
  return upgrades.deployProxy(await new AaveYield__factory().connect(owner), [aavePoolAddress, allowedTokens], {
    initializer: 'initialize',
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

export async function deployTestLidoInteractor(
  owner: SignerWithAddress,
  weth: string,
  stEth: string,
  lidoWithdrawalQueue: string
): Promise<TestLidoInteractor> {
  return upgrades.deployProxy(new TestLidoInteractor__factory().connect(owner), [weth, stEth, lidoWithdrawalQueue], {
    initializer: 'initialize',
  }) as unknown as Promise<TestLidoInteractor>;
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
  return upgrades.deployProxy(
    new TestEigenLayerInteractor__factory().connect(owner),
    [weth, stEth, elStrategy, elStrategyManager, elDelegationManager, eigenLayerOperator],
    {
      initializer: 'initialize',
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
  weth: TestWETH9;
}> {
  const [owner] = await ethers.getSigners();

  const weth = await new TestWETH9__factory().connect(owner).deploy();

  const testLidoInteractor = await deployTestLidoInteractor(
    owner,
    await weth.getAddress(),
    EthAddressData.stEth,
    EthAddressData.lidoWithdrawalQueue
  );

  const stEth = ERC20__factory.connect(EthAddressData.stEth, owner);
  const lidoWithdrawalQueue = ILidoWithdrawalQueueExtended__factory.connect(EthAddressData.lidoWithdrawalQueue, owner);

  return {
    owner,
    testLidoInteractor,
    lidoWithdrawalQueue,
    stEth,
    weth,
  };
}

export async function testEigenLayerInteractorFixture(): Promise<{
  owner: SignerWithAddress;
  testEigenLayerInteractor: TestEigenLayerInteractor;
  delegationManager: IDelegationManager;
  strategy: IStrategy;
  stEth: ERC20;
  weth: TestWETH9;
}> {
  const [owner] = await ethers.getSigners();

  const weth = await new TestWETH9__factory().connect(owner).deploy();

  const testEigenLayerInteractor = await deployTestEigenLayerInteractor(
    owner,
    await weth.getAddress(),
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
    weth,
  };
}

export interface EthYieldForkTestData {
  weth9: IWETH9;
  stEth: ERC20;
  lidoWithdrawalQueue: ILidoWithdrawalQueueExtended;
  eigenLayerStrategyManager: IStrategyManager;
  eigenLayerStrategy: IStrategy;
  eigenLayerDelegationManager: IDelegationManager;
  eigenLayerOperator: string;
  ethYield: EthYield;
  owner: SignerWithAddress;
}

export async function createEthYieldFork(): Promise<EthYieldForkTestData> {
  const [owner] = await ethers.getSigners();

  const axelarGateway = await deployTestAxelarGateway(owner);
  const ethYield = await deployEthYieldContract(
    owner,
    EthAddressData.stEth,
    EthAddressData.weth,
    EthAddressData.elStrategy,
    EthAddressData.elStrategyManager,
    EthAddressData.elDelegationManager,
    EthAddressData.eigenLayerOperator
  );

  await upgrades.upgradeProxy(ethYield, await new TestEthYield__factory().connect(owner), {
    call: {
      fn: 'initializeV2',
      args: [
        EthAddressData.lidoWithdrawalQueue,
        await axelarGateway.getAddress(),
        EthAddressData.axelarGasService,
        WardenChain,
        WardenContractAddress,
      ],
    },
  });

  const weth9 = IWETH9__factory.connect(EthAddressData.weth, owner);
  const stEth = ERC20__factory.connect(EthAddressData.stEth, owner);
  const eigenLayerStrategyManager = IStrategyManager__factory.connect(EthAddressData.elStrategyManager, owner);
  const eigenLayerStrategy = IStrategy__factory.connect(EthAddressData.elStrategy, owner);
  const eigenLayerDelegationManager = IDelegationManager__factory.connect(EthAddressData.elDelegationManager, owner);
  const lidoWithdrawalQueue = ILidoWithdrawalQueueExtended__factory.connect(EthAddressData.lidoWithdrawalQueue, owner);

  return {
    weth9,
    stEth,
    lidoWithdrawalQueue,
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
  aaveYield: AaveYield;
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
