import { expect, use } from 'chai';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { ethers, upgrades } from 'hardhat';
import { MintableERC20__factory, VaultV1__factory, VaultV2__factory } from '../typechain-types';
import { parseUnits } from 'ethers';

describe('Upgradeable tests', () => {
  it('v1', async () => {
    const [deployer, user] = (await ethers.getSigners());
    const token = await new MintableERC20__factory().connect(deployer).deploy('test token', 'TT');
    const v1 = await upgrades.deployProxy(await new VaultV1__factory().connect(deployer), [await token.getAddress()], {
      initializer: 'initialize',
    });

    const amount = parseUnits("100", 18);
    await token.connect(deployer).mint(user.address, amount);
    await token.connect(user).approve(await v1.getAddress(), amount);

    const userBalanceBefore = await token.balanceOf(user.address);
    const vaultBalanceBefore = await token.balanceOf(v1.getAddress());

    await v1.connect(user).deposit(amount);

    expect(await v1.balance(user.address)).to.be.eq(amount);

    const userBalanceAfter = await token.balanceOf(user.address);
    const vaultBalanceAfter = await token.balanceOf(v1.getAddress());

    console.log(typeof(userBalanceBefore));
    expect(userBalanceBefore - userBalanceAfter).to.be.eq(amount);
    expect(vaultBalanceAfter - vaultBalanceBefore).to.be.eq(amount);
  });

  it('v1 upgrade to v2', async () => {
    const [deployer, user] = (await ethers.getSigners());
    const token = await new MintableERC20__factory().connect(deployer).deploy('test token', 'TT');
    const v1 = await upgrades.deployProxy(await new VaultV1__factory().connect(deployer), [await token.getAddress()], {
      initializer: 'initialize',
    });

    const amount = parseUnits("100", 18);
    await token.connect(deployer).mint(user.address, amount);
    await token.connect(user).approve(await v1.getAddress(), amount);

    await v1.connect(user).deposit(amount);

    const v2 = await upgrades.upgradeProxy(await v1.getAddress(), await new VaultV2__factory().connect(deployer));

    expect(await v2.balance(user.address)).to.be.eq(amount);
    expect(await token.balanceOf(v2.getAddress())).to.be.eq(amount)

    await v2.connect(user).withdraw(amount / 2n);

    expect(await v2.balance(user.address)).to.be.eq(amount / 2n);
    expect(await token.balanceOf(v2.getAddress())).to.be.eq(amount / 2n);
    expect(await token.balanceOf(user.getAddress())).to.be.eq(amount / 2n);
  });
});
