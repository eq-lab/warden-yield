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
  error NotApprovedByGateway();
  error InvalidAddress();
  error InvalidSourceChain();
  error InvalidActionType();
  error AmountTooBig();

  uint8 private constant STAKE_ACTION_TYPE = 0;
  uint8 private constant UUNSTAKE_ACTION_TYPE = 1;
  uint8 private constant REINIT_ACTION_TYPE = 2;

  uint8 internal constant SUCCESS_STATUS = 0;
  uint8 internal constant ERROR_STATUS = 1;

  /// keccak256(abi.encode(uint256(keccak256("eq-lab.storage.WardenHandlerData")) - 1)) & ~bytes32(uint256(0xff));
  bytes32 private constant WardenHandlerDataStorageLocation =
    0x4f376997038d6e5610d23f9f89ae844faaf6e156ed92caa3ff61a3cac093a900;

  struct WardenHandlerData {
    address axelarGateway;
    address axelarGasService;
    string wardenChain;
    string wardenContractAddress;
  }

  struct WardenRequest {
    uint8 actionType;
    uint64 actionId;
    uint128 lpAmount;
  }

  struct StakeResult {
    uint8 status;
    uint128 lpAmount;
    uint128 tokenAmount;
    uint64 reinitUnstakeId;
  }

  struct UnstakeResult {
    uint8 status;
    address tokenAddress;
    uint128 tokenAmount;
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
    string calldata wardenChain,
    string calldata wardenContractAddress
  ) internal onlyInitializing {
    if (axelarGateway == address(0)) revert InvalidAddress();

    WardenHandlerData storage $ = _getWardenHandlerData();
    $.axelarGateway = axelarGateway;
    $.axelarGasService = axelarGasService;
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
        actionType: (uint8)(wardenRequest),
        actionId: (uint64)(wardenRequest >> 8),
        lpAmount: (uint128)(wardenRequest >> 72)
      });
  }

  /// @notice Encode warden payload
  function _createResponse(bytes memory argValues) private pure returns (bytes memory) {
    string[] memory argNameArray = new string[](1);
    argNameArray[0] = 'response_data';

    string[] memory argTypeArray = new string[](1);
    argTypeArray[0] = 'bytes';

    bytes memory gmpPayload;
    gmpPayload = abi.encode('handle_response', argNameArray, argTypeArray, argValues);

    return abi.encodePacked(bytes4(0x00000001), gmpPayload);
  }

  /// @notice Encode stake response
  /// @param stakeId Stake identifier
  /// @param stakeResult Stake result
  function _createStakeResponse(uint64 stakeId, StakeResult memory stakeResult) private pure returns (bytes memory) {
    return
      _createResponse(
        abi.encodePacked(
          STAKE_ACTION_TYPE,
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
    uint8 status,
    uint64 unstakeId,
    uint64 reinitUnstakeId
  ) private pure returns (bytes memory) {
    return _createResponse(abi.encodePacked(UUNSTAKE_ACTION_TYPE, status, unstakeId, reinitUnstakeId));
  }

  /// @notice Encode reinit response
  /// @param reinitUnstakeId Reinited unstake identifier
  function _createReinitResponse(uint64 reinitUnstakeId) private pure returns (bytes memory) {
    return _createResponse(abi.encodePacked(REINIT_ACTION_TYPE, reinitUnstakeId));
  }

  ///@notice Handle stake request, should be implemented in Yield contract
  ///@param stakeId Stake identifier
  ///@param tokenAddress Address of the token
  ///@param amountToStake Amount of tokens to stake
  /// @return Stake reesult
  function _handleStakeRequest(
    uint64 stakeId,
    address tokenAddress,
    uint256 amountToStake
  ) internal virtual returns (StakeResult memory);

  /// @notice Handle unstake request, should be implemented in Yield contract
  /// @param unstakeId Unstake identifier
  /// @param lpAmount Amount of lp token to unstake
  /// @return ustake result
  function _handleUnstakeRequest(uint64 unstakeId, uint128 lpAmount) internal virtual returns (UnstakeResult memory);

  ///@notice Handle reinit request, should be implemented in Yield contract
  /// @return reinit result
  function _handleReinitRequest() internal virtual returns (ReinitResult memory);

  ///@notice Axelar relayer calls the function when accept unstake request from Warden
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

    if (request.actionType == UUNSTAKE_ACTION_TYPE) {
      UnstakeResult memory unstakeResult = _handleUnstakeRequest(request.actionId, request.lpAmount);

      tokenAddress = unstakeResult.tokenAddress;
      tokenAmount = unstakeResult.tokenAmount;
      response = _createUnstakeResponse(unstakeResult.status, request.actionId, unstakeResult.reinitUnstakeId);
    } else if (request.actionType == REINIT_ACTION_TYPE) {
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
      IERC20(tokenAddress).approve(address(gateway), tokenAmount);
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
    if (request.actionType != STAKE_ACTION_TYPE) revert InvalidActionType();

    address tokenAddress = gateway.tokenAddresses(tokenSymbol);
    StakeResult memory stakeResult = _handleStakeRequest(request.actionId, tokenAddress, amount);

    // Response to Warden
    bytes memory response = _createStakeResponse(request.actionId, stakeResult);

    // amount to return could contain withdrawn amount and stake amount when stake failed
    if (stakeResult.status != SUCCESS_STATUS) {
      stakeResult.tokenAmount += uint128(amount); // safe cast, checked above
    }

    if (stakeResult.tokenAmount != 0) {
      /// When stake fails, we need to send tokens back to Warden
      IERC20(tokenAddress).approve(address(gateway), stakeResult.tokenAmount);
      gateway.callContractWithToken(wardenChain, wardenContractAddress, response, tokenSymbol, stakeResult.tokenAmount);
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
      msg.sender,
      wardenChain,
      wardenContractAddress,
      response,
      tokenSymbol,
      reinitResult.tokenAmount,
      msg.sender
    );

    IAxelarGateway gateway = IAxelarGateway($.axelarGateway);
    IERC20(reinitResult.tokenAddress).approve(address(gateway), reinitResult.tokenAmount);
    gateway.callContractWithToken(wardenChain, wardenContractAddress, response, tokenSymbol, reinitResult.tokenAmount);
  }
}
