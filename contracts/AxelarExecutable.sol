// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol';
import './interfaces/IAxelarGateway.sol';

/// @notice Holds axelar gateway and gas service addresses and handles interactions from other chains
/// @dev To interact with other chains, use Gateway and gasService.
/// To handle interactions from other chains, implement the _execute and _executeWithToken methods.
abstract contract AxelarExecutable is Initializable {
  error NotApprovedByGateway();
  error InvalidAddress();
  error InvalidWardenAction();

  uint8 private constant STAKE_ACTION_TYPE = 0;
  uint8 private constant UUNSTAKE_ACTION_TYPE = 1;

  uint8 private constant SUCCESS_STATUS = 0;
  uint8 private constant ERROR_STATUS = 1;

  struct AxelarData {
    address gateway;
  }

  /// keccak256(abi.encode(uint256(keccak256("eq-lab.storage.AxelarData")) - 1)) & ~bytes32(uint256(0xff));
  bytes32 private constant AxelarDataStorageLocation =
    0x425f6b37f295fd163c4560fd76ae5ecda01050f2bd962b6c5d8a382923214c00;

  /// @notice Initialize AxelarExecutable module
  /// @param gateway Address of Axelar gateway
  function __AxelarExecutable_init(address gateway) internal onlyInitializing {
    require(gateway != address(0), InvalidAddress());

    AxelarData storage $ = _getAxelarData();
    $.gateway = gateway;
  }

  function _getAxelarData() private pure returns (AxelarData storage $) {
    assembly {
      $.slot := AxelarDataStorageLocation
    }
  }

  ///@notice Get address of Axelar gateway
  function getAxelarGateway() external view returns (address) {
    AxelarData storage $ = _getAxelarData();
    return $.gateway;
  }

  /// @notice Extracts sharesAmount, actionId and actionType from wardenRequest value
  function _decodeWardenPayload(
    bytes calldata payload
  ) private pure returns (uint8 actionType, uint64 actionId, uint128 sharesAmount) {
    uint256 wardenRequest = abi.decode(payload, (uint256));
    actionType = (uint8)(wardenRequest);
    actionId = (uint64)(wardenRequest >> 8);
    sharesAmount = (uint128)(wardenRequest >> 72);
  }

  /// @notice Encode sharesAmount, actionId and status into single uint256 value
  function encodeResponsePayload(
    uint8 status,
    uint64 actionId,
    uint128 sharesAmount
  ) private pure returns (uint256 actionResponse) {
    actionResponse += sharesAmount;
    actionResponse = actionResponse << 64; // 64 bits for actionId
    actionResponse += uint256(actionId);
    actionResponse = actionResponse << 8; // 8 bits for status
    actionResponse += status;
  }

  /// @notice Encode data for execution message 'try_handle_stake_response' in the CosmWasm
  /// @param actionId  Action identifier received from the Warden CosmWasm
  /// @param status Status of stake 1 - Success, 2 - Error
  /// @param sharesAmount Amount of shares, lp token that should be mint to the user in the Warden
  function _createResponse(
    uint64 actionId,
    uint8 status,
    uint128 sharesAmount,
    string memory argName,
    string memory methodName
  ) private pure returns (bytes memory) {
    string[] memory argNameArray = new string[](1);
    argNameArray[0] = argName;

    string[] memory argTypeArray = new string[](1);
    argTypeArray[0] = 'uint256';

    uint256 stakeResponse = encodeResponsePayload(status, actionId, sharesAmount);

    bytes memory argValues = abi.encode(stakeResponse);

    bytes memory gmpPayload;
    gmpPayload = abi.encode(methodName, argNameArray, argTypeArray, argValues);

    return abi.encodePacked(bytes4(0x00000001), gmpPayload);
  }

  function _createStakeResponse(
    uint64 actionId,
    uint8 status,
    uint128 sharesAmount
  ) private pure returns (bytes memory) {
    return _createResponse(actionId, status, sharesAmount, 'stake_response', 'handle_stake_response');
  }

  function _createUnstakeResponse(uint64 actionId, uint8 status) private pure returns (bytes memory) {
    return _createResponse(actionId, status, 0, 'unstake_response', 'handle_unstake_response');
  }

  ///@notice Handle stake request, should be implemented in Yield contract
  ///@param stakeId Stake identifier
  ///@param tokenAddress Address of the token
  ///@param amount Amount of tokens to stake
  function _handleStakeRequest(
    uint64 stakeId,
    address tokenAddress,
    uint256 amount
  ) internal virtual returns (uint8 status, uint128 sharesAmount) {}

  /// @notice Handle unstake request, should be implemented in Yield contract
  /// @param unstakeId Unstake identifier
  /// @param sharesAmount Amount of lp token to unstake
  function _handleUnstakeRequest(
    uint64 unstakeId,
    uint128 sharesAmount
  ) internal virtual returns (uint8 status, address tokenAddress, uint128 tokenAmount) {}

  ///@notice Axelar relayer calls the function when accept unstake request from Warden
  function execute(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes calldata payload
  ) external {
    AxelarData storage $ = _getAxelarData();
    IAxelarGateway gateway = IAxelarGateway($.gateway);
    if (!gateway.validateContractCall(commandId, sourceChain, sourceAddress, keccak256(payload)))
      revert NotApprovedByGateway();

    //TODO: validate sourceChain and sourceAddress

    (uint8 actionType, uint64 actionId, uint128 sharesAmount) = _decodeWardenPayload(payload);
    if (actionType != UUNSTAKE_ACTION_TYPE) revert InvalidWardenAction();

    (uint8 status, address tokenAddress, uint128 tokenAmount) = _handleUnstakeRequest(actionId, sharesAmount);

    // Response to Warden
    bytes memory stakeResponse = _createUnstakeResponse(actionId, status);
    string memory destinationChain = 'warden';
    string memory contractAddress = 'warden-contract-address'; //TODO: get Warden contract address

    if (status == SUCCESS_STATUS && tokenAmount != 0) {
      // When unstake succeeds, we need to send tokens back to Warden
      IERC20(tokenAddress).approve(address(gateway), tokenAmount);
      string memory tokenSymbol = IERC20Metadata(tokenAddress).symbol();
      gateway.callContractWithToken(destinationChain, contractAddress, stakeResponse, tokenSymbol, tokenAmount);
    } else {
      /// When stake fails, we need to send tokens back to Warden
      gateway.callContract(destinationChain, contractAddress, stakeResponse);
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
    AxelarData storage $ = _getAxelarData();
    IAxelarGateway gateway = IAxelarGateway($.gateway);
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

    //TODO: validate sourceChain and sourceAddress

    (uint8 actionType, uint64 actionId, ) = _decodeWardenPayload(payload);
    if (actionType != STAKE_ACTION_TYPE) revert InvalidWardenAction();

    address tokenAddress = gateway.tokenAddresses(tokenSymbol);
    (uint8 status, uint128 sharesAmount) = _handleStakeRequest(actionId, tokenAddress, amount);

    // Response to Warden
    bytes memory stakeResponse = _createStakeResponse(actionId, status, sharesAmount);
    string memory destinationChain = 'warden';
    string memory contractAddress = 'warden-contract-address'; //TODO: get Warden contract address

    if (status == SUCCESS_STATUS) {
      gateway.callContract(destinationChain, contractAddress, stakeResponse);
    } else {
      /// When stake fails, we need to send tokens back to Warden
      IERC20(tokenAddress).approve(address(gateway), amount);
      gateway.callContractWithToken(destinationChain, contractAddress, stakeResponse, tokenSymbol, amount);
    }
  }
}
