import { ethers } from 'hardhat';
import { ILidoWithdrawalQueueExtended } from '../../typechain-types';
import { BytesLike } from 'ethers';
import * as helpers from '@nomicfoundation/hardhat-network-helpers';
import { ActionType } from './warden-handler-fixtures';

export const USER_WARDEN_ADDRESS = 'warden1234';

export const WardenChain = 'warden';
export const WardenContractAddress = 'warden-cosm-wasm-address';

export enum EthAddressData {
  weth = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
  stEth = '0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84',
  wstEth = '0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0',
  lidoWithdrawalQueue = '0x889edC2eDab5f40e902b864aD4d7AdE8E412F9B1',
  aEth = '0x4d5F47FA6A74757f35C14fD3a6Ef8E3C9BC514E8',
  usdt = '0xdAC17F958D2ee523a2206206994597C13D831ec7',
  aEthUsdt = '0x23878914EFE38d27C4D67Ab83ed1b93A74D4086a',
  usdc = '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  aEthUsdc = '0x98C23E9d8f34FEFb1B7BD6a91B7FF122F4e16F5c',
  aaveEthPool = '0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2',
  elStrategyManager = '0x858646372CC42E1A627fcE94aa7A7033e7CF075A',
  elStrategy = '0x93c4b944D05dfe6df7645A86cd2206016c51564D',
  elDelegationManager = '0x39053D51B77DC0d36036Fc1fCc8Cb819df8Ef37A',
  eigenLayerOperator = '0x71C6F7Ed8C2d4925d0bAf16f6A85BB1736D412eb',
  axelarGateway = '0x4F4495243837681061C4743b74B3eEdf548D56A5',
  axelarGasService = '0x2d5d7d31F671F86C782533cc367F14109a082712',
  ethYield = '0x4DF66BCA96319C6A033cfd86c38BCDb9B3c11a72',
  aaveYieldUsdc = '0x0259044395FE54d8aFe28354Ac737EB216064cF9',
  aaveYieldUsdt = '0x0F9d2C03AD21a30746A4b4f07919e1C5F3641F35',
}

export const TokenBalanceStorage: Map<string, string> = new Map([
  [EthAddressData.weth, '0000000000000000000000000000000000000000000000000000000000000003'],
  [EthAddressData.usdt, '0000000000000000000000000000000000000000000000000000000000000002'],
  [EthAddressData.usdc, '0000000000000000000000000000000000000000000000000000000000000009'],
]);

export async function finalizeLidoWithdraw(lidoWithdrawalQueue: ILidoWithdrawalQueueExtended, requestId: bigint) {
  const finalizeRole = await lidoWithdrawalQueue.FINALIZE_ROLE();
  const finalizerAddress = await lidoWithdrawalQueue.getRoleMember(finalizeRole, 0);
  const impersonatedSigner = await ethers.getImpersonatedSigner(finalizerAddress);
  const maxShares = 10n ** 50n; // passing some unrealistic value for 1e27 precision
  const ethersToFinalize = await lidoWithdrawalQueue.unfinalizedStETH();
  if ((ethersToFinalize * 11n) / 10n > (await impersonatedSigner.provider.getBalance(finalizerAddress))) {
    await helpers.setBalance(finalizerAddress, (ethersToFinalize * 11n) / 10n);
  }
  await lidoWithdrawalQueue.connect(impersonatedSigner).finalize(requestId, maxShares, { value: ethersToFinalize });
}

function getAccountBalanceStorageSlot(account: string, tokenMappingSlot: string): string {
  if (!ethers.isAddress(account)) {
    throw new Error(`failed to get token balance: wrong address ${account}`);
  }
  return ethers.keccak256('0x' + account.slice(2).padStart(64, '0') + tokenMappingSlot);
}

export async function setTokenBalance(tokenAddress: string, account: string, newBalance: bigint) {
  if (!ethers.isAddress(account)) {
    throw new Error(`failed to set token balance: wrong address ${account}`);
  }

  const balanceOfSlotAddress = TokenBalanceStorage.get(tokenAddress);
  if (balanceOfSlotAddress === undefined) {
    throw new Error(`unknown storage slot for token ${tokenAddress}`);
  }
  const balanceOfStorageSlot = getAccountBalanceStorageSlot(account, balanceOfSlotAddress);

  await ethers.provider.send('hardhat_setStorageAt', [
    tokenAddress,
    balanceOfStorageSlot,
    ethers.zeroPadValue(toHexString(newBalance), 32),
  ]);
}

function toHexString(value: bigint): string {
  let hexString: string = value.toString(16);

  if (hexString.length % 2 !== 0) {
    hexString = '0' + hexString;
  }
  return `0x${hexString}`;
}

export function encodeStakeAction(stakeId: number): BytesLike {
  return encodeWardenPayload(ActionType.Stake, stakeId, 0n);
}

export function encodeUnstakeAction(unstakeId: number, lpAmount: bigint): BytesLike {
  return encodeWardenPayload(ActionType.Unstake, unstakeId, lpAmount);
}

export function encodeReinitAction(): BytesLike {
  return encodeWardenPayload(ActionType.Reinit, 0, 0n);
}

export function encodeWardenPayload(actionType: number, actionId: number, lpAmount: bigint): BytesLike {
  let value = 0n;
  value += lpAmount << 72n;
  value += BigInt(actionId) << 8n;
  value += BigInt(actionType);
  return ethers.solidityPacked(['uint256'], [value]);
}

type WardenStakeResponse = {
  actionType: number;
  status: number;
  actionId: number;
  lpAmount: bigint;
  reinitUnstakeId: number;
};

function decodeArgValues(payload: BytesLike): string {
  const abiCoder = ethers.AbiCoder.defaultAbiCoder();
  const gmpPayload = ethers.dataSlice(payload, 4);

  const decoded = abiCoder.decode(['string', 'string[]', 'string[]', 'bytes'], gmpPayload, false);
  const abiEncodedArgs = decoded.at(3);
  const args = abiCoder.decode(['bytes'], abiEncodedArgs).at(0);
  return args;
}

export function decodeWardenStakeResponse(payload: BytesLike): WardenStakeResponse {
  const argValues = decodeArgValues(payload);

  return {
    actionType: ethers.getNumber(ethers.dataSlice(argValues, 0, 1)), //actionType 1 byte
    status: ethers.getNumber(ethers.dataSlice(argValues, 1, 2)), // status 1 byte
    actionId: ethers.getNumber(ethers.dataSlice(argValues, 2, 10)), //actionId 8 byte
    reinitUnstakeId: ethers.getNumber(ethers.dataSlice(argValues, 10, 18)), //reinitUnstakeId 8 byte
    lpAmount: ethers.getBigInt(ethers.dataSlice(argValues, 18, 34)), //lpAmount 16 byte
  };
}

type WardenUnstakeResponse = {
  actionType: number;
  status: number;
  actionId: number;
  reinitUnstakeId: number;
};

export function decodeWardenUnstakeResponse(payload: BytesLike): WardenUnstakeResponse {
  const argValues = decodeArgValues(payload);

  return {
    actionType: ethers.getNumber(ethers.dataSlice(argValues, 0, 1)), //actionType 1 byte
    status: ethers.getNumber(ethers.dataSlice(argValues, 1, 2)), // status 1 byte
    actionId: ethers.getNumber(ethers.dataSlice(argValues, 2, 10)), //actionId 8 byte
    reinitUnstakeId: ethers.getNumber(ethers.dataSlice(argValues, 10, 18)), //reinitUnstakeId 8 byte
  };
}

type WardenReinitResponse = {
  actionType: number;
  reinitUnstakeId: number;
};

export function decodeWardenReinitResponse(payload: BytesLike): WardenReinitResponse {
  const argValues = decodeArgValues(payload);
  return {
    actionType: ethers.getNumber(ethers.dataSlice(argValues, 0, 1)), //actionType 1 byte
    reinitUnstakeId: ethers.getNumber(ethers.dataSlice(argValues, 1, 9)), //reinitUnstakeId 8 byte
  };
}
