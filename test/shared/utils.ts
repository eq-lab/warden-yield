import { ethers } from 'hardhat';

export const TokenBalanceStorage: Map<string, string> = new Map([
  ['0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2', '0000000000000000000000000000000000000000000000000000000000000003'], // weth9
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
