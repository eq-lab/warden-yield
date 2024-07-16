import { expect } from 'chai';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { parseUnits } from 'ethers';
import {
  testWardentHandlerFixture,
  WardenContractAddress,
  CommandId,
  WardenChain,
  ActionType,
  Status,
} from './shared/warden-handler-fixtures';
import {
  decodeWardenReinitResponse,
  decodeWardenStakeResponse,
  decodeWardenUnstakeResponse,
  encodeWardenPayload,
} from './shared/utils';

describe('WardenHandler', () => {
  describe('stake request', () => {
    it('Should fail when wrong source chain', async () => {
      const { wardenHandler, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = 'WrongChain';
      const sourceContractAddress = WardenContractAddress;
      const payload = '0x0000';
      const tokenSymbol = await testToken.symbol();
      const tokenAmount = parseUnits('1', 18);

      await expect(
        wardenHandler.executeWithToken(CommandId, sourceChain, sourceContractAddress, payload, tokenSymbol, tokenAmount)
      ).to.be.revertedWithCustomError(wardenHandler, 'InvalidSourceChain');
    });

    it('Should fail when wrong source contract address', async () => {
      const { wardenHandler, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = 'wrong-contract-address';
      const payload = '0x0000';
      const tokenSymbol = await testToken.symbol();
      const tokenAmount = parseUnits('1', 18);

      await expect(
        wardenHandler.executeWithToken(CommandId, sourceChain, sourceContractAddress, payload, tokenSymbol, tokenAmount)
      ).to.be.revertedWithCustomError(wardenHandler, 'InvalidSourceChain');
    });

    it('Should fail when not approved by gateway', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);

      await axelarGateway.setIsValidCall(false);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = '0x0000';
      const tokenSymbol = await testToken.symbol();
      const tokenAmount = parseUnits('1', 18);

      await expect(
        wardenHandler.executeWithToken(CommandId, sourceChain, sourceContractAddress, payload, tokenSymbol, tokenAmount)
      ).to.be.revertedWithCustomError(wardenHandler, 'NotApprovedByGateway');
    });

    it('Should fail when wrong action type', async () => {
      const { wardenHandler, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = encodeWardenPayload(ActionType.Unstake, 0, 0n);
      const tokenSymbol = await testToken.symbol();
      const tokenAmount = parseUnits('1', 18);

      await expect(
        wardenHandler.executeWithToken(CommandId, sourceChain, sourceContractAddress, payload, tokenSymbol, tokenAmount)
      ).to.be.revertedWithCustomError(wardenHandler, 'InvalidActionType');
    });

    it('Should call staking and reply success', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = encodeWardenPayload(ActionType.Stake, 1, 0n);
      const tokenSymbol = await testToken.symbol();
      const tokenAmount = parseUnits('1', 18);

      const stakeResult = {
        actionType: ActionType.Stake,
        status: Status.Success,
        unstakeTokenAmount: 0n,
        reinitUnstakeId: 0n,
        lpAmount: 1000n,
        actionId: 1,
      };
      await wardenHandler.setStakeResult(stakeResult);

      await wardenHandler.executeWithToken(
        CommandId,
        sourceChain,
        sourceContractAddress,
        payload,
        tokenSymbol,
        tokenAmount
      );

      const wardenRsponse = await axelarGateway.callContractPayload();

      const stakeResponse = decodeWardenStakeResponse(wardenRsponse);
      expect(stakeResponse.status).to.equal(stakeResult.status);
      expect(stakeResponse.actionType).to.equal(stakeResult.actionType);
      expect(stakeResponse.actionId).to.equal(stakeResult.actionId);
      expect(stakeResponse.lpAmount).to.equal(stakeResult.lpAmount);
      expect(stakeResponse.reinitUnstakeId).to.equal(stakeResult.reinitUnstakeId);
    });

    it('Should call staking and reply failed', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = encodeWardenPayload(ActionType.Stake, 1, 0n);
      const tokenSymbol = await testToken.symbol();
      const tokenAmount = parseUnits('1', 18);

      const stakeResult = {
        actionType: ActionType.Stake,
        actionId: 1,
        status: Status.Failed,
        unstakeTokenAmount: 1000n,
        reinitUnstakeId: 0n,
        lpAmount: 0n,
      };
      await wardenHandler.setStakeResult(stakeResult);

      await wardenHandler.executeWithToken(
        CommandId,
        sourceChain,
        sourceContractAddress,
        payload,
        tokenSymbol,
        tokenAmount
      );

      const wardenRsponse = await axelarGateway.callContractWithTokenPayload();

      const stakeResponse = decodeWardenStakeResponse(wardenRsponse);
      expect(stakeResponse.status).to.equal(stakeResult.status);
      expect(stakeResponse.actionType).to.equal(stakeResult.actionType);
      expect(stakeResponse.actionId).to.equal(stakeResult.actionId);
      expect(stakeResponse.lpAmount).to.equal(stakeResult.lpAmount);
      expect(stakeResponse.reinitUnstakeId).to.equal(stakeResult.reinitUnstakeId);
    });

    it('Should call staking, reply success with reinitUnstakeId', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = encodeWardenPayload(ActionType.Stake, 1, 0n);
      const tokenSymbol = await testToken.symbol();
      const tokenAmount = parseUnits('1', 18);

      const stakeResult = {
        actionType: ActionType.Stake,
        actionId: 1,
        status: Status.Success,
        unstakeTokenAmount: 1000n,
        reinitUnstakeId: 1n,
        lpAmount: 0n,
      };
      await wardenHandler.setStakeResult(stakeResult);

      await wardenHandler.executeWithToken(
        CommandId,
        sourceChain,
        sourceContractAddress,
        payload,
        tokenSymbol,
        tokenAmount
      );

      const wardenRsponse = await axelarGateway.callContractWithTokenPayload();

      const stakeResponse = decodeWardenStakeResponse(wardenRsponse);
      expect(stakeResponse.status).to.equal(stakeResult.status);
      expect(stakeResponse.actionType).to.equal(stakeResult.actionType);
      expect(stakeResponse.actionId).to.equal(stakeResult.actionId);
      expect(stakeResponse.lpAmount).to.equal(stakeResult.lpAmount);
      expect(stakeResponse.reinitUnstakeId).to.equal(stakeResult.reinitUnstakeId);
    });
  });

  describe('Unstake and reinit requests', () => {
    it('Should fail when wrong source chain', async () => {
      const { wardenHandler, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = 'WrongChain';
      const sourceContractAddress = WardenContractAddress;
      const payload = '0x0000';

      await expect(
        wardenHandler.execute(CommandId, sourceChain, sourceContractAddress, payload)
      ).to.be.revertedWithCustomError(wardenHandler, 'InvalidSourceChain');
    });

    it('Should fail when wrong source contract address', async () => {
      const { wardenHandler, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = 'wrong-contract-address';
      const payload = '0x0000';

      await expect(
        wardenHandler.execute(CommandId, sourceChain, sourceContractAddress, payload)
      ).to.be.revertedWithCustomError(wardenHandler, 'InvalidSourceChain');
    });

    it('Should fail when not approved by gateway', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);

      await axelarGateway.setIsValidCall(false);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = '0x0000';

      await expect(
        wardenHandler.execute(CommandId, sourceChain, sourceContractAddress, payload)
      ).to.be.revertedWithCustomError(wardenHandler, 'NotApprovedByGateway');
    });

    it('Should fail when wrong action type', async () => {
      const { wardenHandler, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = encodeWardenPayload(ActionType.Stake, 0, 0n);

      await expect(
        wardenHandler.execute(CommandId, sourceChain, sourceContractAddress, payload)
      ).to.be.revertedWithCustomError(wardenHandler, 'InvalidActionType');
    });

    it('Should call unstake and reply success', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const unstakeId = 1;
      const lpAmount = parseUnits('1', 18);
      const payload = encodeWardenPayload(ActionType.Unstake, unstakeId, lpAmount);
      const tokenAmount = parseUnits('1', 18);

      const unstakeResult = {
        status: Status.Success,
        reinitUnstakeId: 0n,
        unstakeTokenAmount: tokenAmount,
        unstakeTokenAddress: await testToken.getAddress(),
      };
      await wardenHandler.setUnstakeResult(unstakeResult);

      await wardenHandler.execute(CommandId, sourceChain, sourceContractAddress, payload);

      const wardenRsponse = await axelarGateway.callContractWithTokenPayload();

      const unstakeResponse = decodeWardenUnstakeResponse(wardenRsponse);
      expect(unstakeResponse.status).to.equal(unstakeResult.status);
      expect(unstakeResponse.actionType).to.equal(ActionType.Unstake);
      expect(unstakeResponse.actionId).to.equal(unstakeId);
      expect(unstakeResponse.reinitUnstakeId).to.equal(unstakeResult.reinitUnstakeId);
    });

    it('Should call reinit and reply success', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = encodeWardenPayload(ActionType.Reinit, 0, 0n);
      const tokenAmount = parseUnits('1', 18);

      const reinitResult = {
        reinitUnstakeId: 1n,
        tokenAmount: tokenAmount,
        tokenAddress: await testToken.getAddress(),
      };
      await wardenHandler.setReinitResult(reinitResult);

      await wardenHandler.execute(CommandId, sourceChain, sourceContractAddress, payload);

      const wardenRsponse = await axelarGateway.callContractWithTokenPayload();

      const reinitResponse = decodeWardenReinitResponse(wardenRsponse);
      expect(reinitResponse.actionType).to.equal(ActionType.Reinit);
      expect(reinitResponse.reinitUnstakeId).to.equal(reinitResult.reinitUnstakeId);
    });

    it('Should call reinit with empty reply', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);
      await axelarGateway.resetPayload();

      expect(await axelarGateway.callContractPayload()).to.eq('0x');
      expect(await axelarGateway.callContractWithTokenPayload()).to.eq('0x');

      const sourceChain = WardenChain;
      const sourceContractAddress = WardenContractAddress;
      const payload = encodeWardenPayload(ActionType.Reinit, 0, 0n);

      const reinitResult = {
        reinitUnstakeId: 0n,
        tokenAmount: 0n,
        tokenAddress: await testToken.getAddress(),
      };
      await wardenHandler.setReinitResult(reinitResult);
      await wardenHandler.execute(CommandId, sourceChain, sourceContractAddress, payload);

      //ensure no response to Warden
      expect(await axelarGateway.callContractPayload()).to.eq('0x');
      expect(await axelarGateway.callContractWithTokenPayload()).to.eq('0x');
    });
  });

  describe('Reinit from EVM', () => {
    it('Should reinit and reply to Warden', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);
      await axelarGateway.resetPayload();

      expect(await axelarGateway.callContractPayload()).to.eq('0x');
      expect(await axelarGateway.callContractWithTokenPayload()).to.eq('0x');

      const tokenAmount = parseUnits('1', 18);

      const reinitResult = {
        reinitUnstakeId: 1n,
        tokenAmount: tokenAmount,
        tokenAddress: await testToken.getAddress(),
      };
      await wardenHandler.setReinitResult(reinitResult);
      await wardenHandler.executeReinit();

      const wardenRsponse = await axelarGateway.callContractWithTokenPayload();
      const reinitResponse = decodeWardenReinitResponse(wardenRsponse);
      expect(reinitResponse.actionType).to.equal(ActionType.Reinit);
      expect(reinitResponse.reinitUnstakeId).to.equal(reinitResult.reinitUnstakeId);
    });

    it('Should reinit without reply to Warden', async () => {
      const { wardenHandler, axelarGateway, testToken } = await loadFixture(testWardentHandlerFixture);
      await axelarGateway.resetPayload();

      expect(await axelarGateway.callContractPayload()).to.eq('0x');
      expect(await axelarGateway.callContractWithTokenPayload()).to.eq('0x');

      const reinitResult = {
        reinitUnstakeId: 0n,
        tokenAmount: 0n,
        tokenAddress: await testToken.getAddress(),
      };
      await wardenHandler.setReinitResult(reinitResult);
      await wardenHandler.executeReinit();

      //ensure no response to Warden
      expect(await axelarGateway.callContractPayload()).to.eq('0x');
      expect(await axelarGateway.callContractWithTokenPayload()).to.eq('0x');
    });
  });
});
