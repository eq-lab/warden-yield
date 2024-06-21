import { ethers } from 'hardhat';

export enum EthAddressData {
  weth = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
  stEth = '0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84',
  aEth = '0x4d5F47FA6A74757f35C14fD3a6Ef8E3C9BC514E8',
  aaveEthPool = '0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2',
  elStrategyManager = '0x858646372CC42E1A627fcE94aa7A7033e7CF075A',
  elStrategy = '0x93c4b944D05dfe6df7645A86cd2206016c51564D',
  elDelegationManager = '0x39053D51B77DC0d36036Fc1fCc8Cb819df8Ef37A',
  eigenLayerOperator = '0x71C6F7Ed8C2d4925d0bAf16f6A85BB1736D412eb',
}

export const TokenBalanceStorage: Map<string, string> = new Map([
  [EthAddressData.weth, '0000000000000000000000000000000000000000000000000000000000000003'],
]);

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
