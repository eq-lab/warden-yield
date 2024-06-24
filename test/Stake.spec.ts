import { expect } from 'chai';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers } from 'hardhat';
import { parseUnits } from 'ethers';
import { testYieldStorageFixture } from './shared/fixtures';
import { USER_WARDEN_ADDRESS } from './shared/utils';

describe('Staking', () => {
  it('stake', async () => {
    const { testYieldStorage, weth9 } = await loadFixture(testYieldStorageFixture);
    const weth9Address = await weth9.getAddress();
    const [_, user] = await ethers.getSigners();
    const stakeAmount = parseUnits('10', 18);
    await testYieldStorage.connect(user).stake(stakeAmount);

    const shares = await testYieldStorage.getStakedAmount(stakeAmount);

    expect(await testYieldStorage.userStakedAmount(user.address, weth9Address)).to.be.eq(stakeAmount);
    expect(await testYieldStorage.userShares(user.address, weth9Address)).to.be.eq(shares);
    expect(await testYieldStorage.totalStakedAmount(weth9Address)).to.be.eq(stakeAmount);
    expect(await testYieldStorage.totalShares(weth9Address)).to.be.eq(shares);
  });
});

describe('AddWardenAddress', () => {
  it('add new warden address', async () => {
    const { testYieldStorage } = await loadFixture(testYieldStorageFixture);
    const [_, user] = await ethers.getSigners();

    await testYieldStorage.connect(user).addWardenAddress(USER_WARDEN_ADDRESS);
    expect(await testYieldStorage.wardenAddress(user.address)).to.be.eq(USER_WARDEN_ADDRESS);
  });

  it('passing the same address after setting', async () => {
    const { testYieldStorage } = await loadFixture(testYieldStorageFixture);
    const [_, user] = await ethers.getSigners();

    await testYieldStorage.connect(user).addWardenAddress(USER_WARDEN_ADDRESS);
    await testYieldStorage.connect(user).addWardenAddress(USER_WARDEN_ADDRESS);
    expect(await testYieldStorage.wardenAddress(user.address)).to.be.eq(USER_WARDEN_ADDRESS);
  });

  it('passing another address after setting', async () => {
    const { testYieldStorage } = await loadFixture(testYieldStorageFixture);
    const [_, user] = await ethers.getSigners();

    await testYieldStorage.connect(user).addWardenAddress(USER_WARDEN_ADDRESS);
    await expect(testYieldStorage.connect(user).addWardenAddress('warden1233')).to.be.revertedWithCustomError(
      testYieldStorage,
      'WrongWardenAddress'
    );
  });
});
