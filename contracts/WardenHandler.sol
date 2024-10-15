// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol';
import '@openzeppelin/contracts/utils/Strings.sol';
import './interfaces/Axelar/IAxelarGateway.sol';
import './interfaces/Axelar/IAxelarGasService.sol';

/// @notice Handles requests from Warden
abstract contract WardenHandler is Initializable {
  using SafeERC20 for IERC20;

  event RequestFailed(ActionType actionType, uint64 actionId, bytes data);

  error NotApprovedByGateway();
  error InvalidAddress();
  error InvalidSourceChain();
  error InvalidActionType();
  error AmountTooBig();
  error Debug(bytes);

  enum ActionType {
    Stake, //0
    Unstake, //1
    Reinit //2
  }

  enum Status {
    Success, //0
    Failed //1
  }

  /// keccak256(abi.encode(uint256(keccak256("eq-lab.storage.WardenHandlerData")) - 1)) & ~bytes32(uint256(0xff));
  bytes32 private constant WardenHandlerDataStorageLocation =
    0x4f376997038d6e5610d23f9f89ae844faaf6e156ed92caa3ff61a3cac093a900;

  /// @custom:storage-location erc7201:eq-lab.storage.WardenHandlerData
  struct WardenHandlerData {
    address axelarGateway;
    address axelarGasService;
    string evmChainName;
    string wardenChain;
    string wardenContractAddress;
  }

  struct WardenRequest {
    ActionType actionType;
    uint64 actionId;
    uint128 lpAmount;
  }

  struct StakeResult {
    Status status;
    uint128 lpAmount;
    uint128 unstakeTokenAmount;
    uint64 reinitUnstakeId;
  }

  struct UnstakeResult {
    Status status;
    address unstakeTokenAddress;
    uint128 unstakeTokenAmount;
    uint64 reinitUnstakeId;
  }

  struct ReinitResult {
    address tokenAddress;
    uint128 tokenAmount;
    uint64 reinitUnstakeId;
  }

  /// @notice Initialize module
  /// @param axelarGateway Address of Axelar gateway
  /// @param axelarGasService Address of Axelar gas service
  /// @param wardenChain Identifier of warden chain
  /// @param wardenContractAddress Contract address in warden chain
  function __WardenHandler_init(
    address axelarGateway,
    address axelarGasService,
    string calldata evmChainName,
    string calldata wardenChain,
    string calldata wardenContractAddress
  ) internal onlyInitializing {
    if (axelarGateway == address(0)) revert InvalidAddress();
    if (axelarGasService == address(0)) revert InvalidAddress();

    WardenHandlerData storage $ = _getWardenHandlerData();
    $.axelarGateway = axelarGateway;
    $.axelarGasService = axelarGasService;
    $.evmChainName = evmChainName;
    $.wardenChain = wardenChain;
    $.wardenContractAddress = wardenContractAddress;
  }

  function _getWardenHandlerData() private pure returns (WardenHandlerData storage $) {
    assembly {
      $.slot := WardenHandlerDataStorageLocation
    }
  }

  /// @notice Extracts lpAmount, actionId and actionType from warden payload
  function _decodeWardenPayload(bytes calldata payload) private pure returns (WardenRequest memory) {
    uint256 wardenRequest = abi.decode(payload, (uint256));
    return
      WardenRequest({
        actionType: ActionType(uint8(wardenRequest)),
        actionId: uint64(wardenRequest >> 8),
        lpAmount: uint128(wardenRequest >> 72)
      });
  }

  /// @notice Encode warden payload
  /// @dev About Evm -> CosmWasm messages https://docs.axelar.dev/dev/cosmos-gmp#messages-from-evm-to-cosmwasm
  function _createResponse(bytes memory argValues) private view returns (bytes memory) {
    WardenHandlerData storage $ = _getWardenHandlerData();
    bytes memory abiEncodedWardenData = abi.encode($.evmChainName, address(this), argValues);
    uint256 dataLength = abiEncodedWardenData.length;
    // if (dataLength > 0) revert Debug(abiEncodedWardenData);

    bytes memory gmpMessage = new bytes(0x360 + dataLength);
    assembly {
      // mstore(0x40, add(gmpMessage, 0x340))
      mstore(add(gmpMessage, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000080)
      mstore(add(gmpMessage, 0x40), 0x00000000000000000000000000000000000000000000000000000000000000c0)
      mstore(add(gmpMessage, 0x60), 0x0000000000000000000000000000000000000000000000000000000000000200)
      mstore(add(gmpMessage, 0x80), 0x0000000000000000000000000000000000000000000000000000000000000340)
      mstore(add(gmpMessage, 0xa0), 0x000000000000000000000000000000000000000000000000000000000000000f)
      mstore(add(gmpMessage, 0xc0), 0x68616e646c655f726573706f6e73650000000000000000000000000000000000)
      mstore(add(gmpMessage, 0xe0), 0x0000000000000000000000000000000000000000000000000000000000000003)
      mstore(add(gmpMessage, 0x100), 0x0000000000000000000000000000000000000000000000000000000000000060)
      mstore(add(gmpMessage, 0x120), 0x00000000000000000000000000000000000000000000000000000000000000a0)
      mstore(add(gmpMessage, 0x140), 0x00000000000000000000000000000000000000000000000000000000000000e0)
      mstore(add(gmpMessage, 0x160), 0x000000000000000000000000000000000000000000000000000000000000000c)
      mstore(add(gmpMessage, 0x180), 0x736f757263655f636861696e0000000000000000000000000000000000000000)
      mstore(add(gmpMessage, 0x1a0), 0x000000000000000000000000000000000000000000000000000000000000000e)
      mstore(add(gmpMessage, 0x1c0), 0x736f757263655f61646472657373000000000000000000000000000000000000)
      mstore(add(gmpMessage, 0x1e0), 0x0000000000000000000000000000000000000000000000000000000000000007)
      mstore(add(gmpMessage, 0x200), 0x7061796c6f616400000000000000000000000000000000000000000000000000)

      mstore(add(gmpMessage, 0x220), 0x0000000000000000000000000000000000000000000000000000000000000003)
      mstore(add(gmpMessage, 0x240), 0x0000000000000000000000000000000000000000000000000000000000000060)
      mstore(add(gmpMessage, 0x260), 0x00000000000000000000000000000000000000000000000000000000000000a0)
      mstore(add(gmpMessage, 0x280), 0x00000000000000000000000000000000000000000000000000000000000000e0)
      mstore(add(gmpMessage, 0x2a0), 0x0000000000000000000000000000000000000000000000000000000000000006)
      mstore(add(gmpMessage, 0x2c0), 0x737472696e670000000000000000000000000000000000000000000000000000)
      mstore(add(gmpMessage, 0x2e0), 0x0000000000000000000000000000000000000000000000000000000000000007)
      mstore(add(gmpMessage, 0x300), 0x6164647265737300000000000000000000000000000000000000000000000000)
      mstore(add(gmpMessage, 0x320), 0x0000000000000000000000000000000000000000000000000000000000000005)
      mstore(add(gmpMessage, 0x340), 0x6279746573000000000000000000000000000000000000000000000000000000)

      for { let i := 0 } lt(i, add(dataLength, 0x20)) { i := add(i, 0x20) } {
        mstore(add(add(gmpMessage, 0x360), i), mload(add(abiEncodedWardenData, i)))
      }

    }
   return abi.encodePacked(uint32(1), gmpMessage);
  }

  /// @notice Encode stake response
  /// @param stakeId Stake identifier
  /// @param stakeResult Stake result
  function _createStakeResponse(uint64 stakeId, StakeResult memory stakeResult) private view returns (bytes memory) {
    return
      _createResponse(
        abi.encodePacked(
          ActionType.Stake,
          stakeResult.status,
          stakeId,
          stakeResult.reinitUnstakeId,
          stakeResult.lpAmount
        )
      );
  }

  /// @notice Encode unstake response
  /// @param status status of unstake
  /// @param unstakeId Unstake identifier
  /// @param reinitUnstakeId Reinited unstake identifier
  function _createUnstakeResponse(
    Status status,
    uint64 unstakeId,
    uint64 reinitUnstakeId
  ) private view returns (bytes memory) {
    return _createResponse(abi.encodePacked(ActionType.Unstake, status, unstakeId, reinitUnstakeId));
  }

  /// @notice Encode reinit response
  /// @param reinitUnstakeId Reinited unstake identifier
  function _createReinitResponse(uint64 reinitUnstakeId) private view returns (bytes memory) {
    return _createResponse(abi.encodePacked(ActionType.Reinit, reinitUnstakeId));
  }

  ///@notice Handle stake request, should be implemented in Yield contract
  ///@param amountToStake Amount of tokens to stake
  /// @return Stake result
  function _handleStakeRequest(uint64 stakeId, uint256 amountToStake) internal virtual returns (StakeResult memory);

  /// @notice Handle unstake request, should be implemented in Yield contract
  /// @param unstakeId Unstake identifier
  /// @param lpAmount Amount of lp token to unstake
  /// @return unstake result
  function _handleUnstakeRequest(uint64 unstakeId, uint128 lpAmount) internal virtual returns (UnstakeResult memory);

  ///@notice Handle reinit request, should be implemented in Yield contract
  /// @return reinit result
  function _handleReinitRequest() internal virtual returns (ReinitResult memory);

  ///@notice Axelar relayer calls the function when accept unstake or reinit request from Warden
  function execute(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes calldata payload
  ) external {
    WardenHandlerData storage $ = _getWardenHandlerData();

    string memory wardenChain = $.wardenChain;
    if (!Strings.equal(wardenChain, sourceChain)) revert InvalidSourceChain();

    string memory wardenContractAddress = $.wardenContractAddress;
    if (!Strings.equal(wardenContractAddress, sourceAddress)) revert InvalidSourceChain();

    IAxelarGateway gateway = IAxelarGateway($.axelarGateway);
    if (!gateway.validateContractCall(commandId, sourceChain, sourceAddress, keccak256(payload)))
      revert NotApprovedByGateway();

    WardenRequest memory request = _decodeWardenPayload(payload);

    address tokenAddress;
    uint128 tokenAmount;
    bytes memory response;

    if (request.actionType == ActionType.Unstake) {
      UnstakeResult memory unstakeResult = _handleUnstakeRequest(request.actionId, request.lpAmount);

      tokenAddress = unstakeResult.unstakeTokenAddress;
      tokenAmount = unstakeResult.unstakeTokenAmount;
      response = _createUnstakeResponse(unstakeResult.status, request.actionId, unstakeResult.reinitUnstakeId);
    } else if (request.actionType == ActionType.Reinit) {
      ReinitResult memory reinitResult = _handleReinitRequest();
      if (reinitResult.tokenAmount == 0) {
        return; // no response for empty reinit
      }

      tokenAddress = reinitResult.tokenAddress;
      tokenAmount = reinitResult.tokenAmount;
      response = _createReinitResponse(reinitResult.reinitUnstakeId);
    } else {
      revert InvalidActionType();
    }

    if (tokenAmount != 0) {
      IERC20(tokenAddress).forceApprove(address(gateway), tokenAmount);
      gateway.callContractWithToken(
        wardenChain,
        wardenContractAddress,
        response,
        IERC20Metadata(tokenAddress).symbol(),
        tokenAmount
      );
    } else {
      gateway.callContract(wardenChain, wardenContractAddress, response);
    }
  }

  ///@notice Axelar relayer calls the function when accept stake request from Warden
  function executeWithToken(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes calldata payload,
    string calldata tokenSymbol,
    uint256 amount
  ) external {
    // Most unlikely case, because Warden token amount is limited by uint128 type
    if (amount > type(uint128).max) revert AmountTooBig();

    WardenHandlerData storage $ = _getWardenHandlerData();

    string memory wardenChain = $.wardenChain;
    if (!Strings.equal(wardenChain, sourceChain)) revert InvalidSourceChain();

    string memory wardenContractAddress = $.wardenContractAddress;
    if (!Strings.equal(wardenContractAddress, sourceAddress)) revert InvalidSourceChain();

    IAxelarGateway gateway = IAxelarGateway($.axelarGateway);
    if (
      !gateway.validateContractCallAndMint(
        commandId,
        sourceChain,
        sourceAddress,
        keccak256(payload),
        tokenSymbol,
        amount
      )
    ) revert NotApprovedByGateway();

    WardenRequest memory request = _decodeWardenPayload(payload);
    if (request.actionType != ActionType.Stake) revert InvalidActionType();

    StakeResult memory stakeResult = _handleStakeRequest(request.actionId, amount);

    // Response to Warden
    bytes memory response = _createStakeResponse(request.actionId, stakeResult);

    // amount to return could contain withdrawn amount and stake amount when stake failed
    // TODO can be moved to `_handleStakeRequest`
    if (stakeResult.status != Status.Success) {
      stakeResult.unstakeTokenAmount += uint128(amount); // safe cast, checked above
    }

    if (stakeResult.unstakeTokenAmount != 0) {
      /// When stake fails, we need to send tokens back to Warden
      address tokenAddress = gateway.tokenAddresses(tokenSymbol);
      IERC20(tokenAddress).forceApprove(address(gateway), stakeResult.unstakeTokenAmount);
      gateway.callContractWithToken(
        wardenChain,
        wardenContractAddress,
        response,
        tokenSymbol,
        stakeResult.unstakeTokenAmount
      );
    } else {
      gateway.callContract(wardenChain, wardenContractAddress, response);
    }
  }

  ///@notice Anyone can call reinit function from EVM chain
  function executeReinit() external payable {
    WardenHandlerData storage $ = _getWardenHandlerData();

    ReinitResult memory reinitResult = _handleReinitRequest();
    if (reinitResult.tokenAmount == 0) {
      return; // no response for empty reinit
    }

    bytes memory response = _createReinitResponse(reinitResult.reinitUnstakeId);
    string memory tokenSymbol = IERC20Metadata(reinitResult.tokenAddress).symbol();
    string memory wardenChain = $.wardenChain;
    string memory wardenContractAddress = $.wardenContractAddress;

    IAxelarGasService($.axelarGasService).payNativeGasForContractCallWithToken{value: msg.value}(
      address(this),
      wardenChain,
      wardenContractAddress,
      response,
      tokenSymbol,
      reinitResult.tokenAmount,
      msg.sender
    );

    IAxelarGateway gateway = IAxelarGateway($.axelarGateway);
    IERC20(reinitResult.tokenAddress).forceApprove(address(gateway), reinitResult.tokenAmount);
    gateway.callContractWithToken(wardenChain, wardenContractAddress, response, tokenSymbol, reinitResult.tokenAmount);
  }
}
