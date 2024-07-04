// SPDX-License-Identifier: GPL-3.0
pragma solidity =0.8.26;

import '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import './interfaces/IAxelarGateway.sol';

/// @notice Holds axelar gateway and gas service addresses and handles interactions from other chains
/// @dev To interact with other chains, use Gateway and gasService.
/// To handle interactions from other chains, implement the _execute and _executeWithToken methods.
abstract contract AxelarExecutable is Initializable {
  error NotApprovedByGateway();
  error InvalidAddress();

  struct AxelarData {
    address gateway;
    address gasService;
  }

  /// keccak256(abi.encode(uint256(keccak256("eq-lab.storage.AxelarData")) - 1)) & ~bytes32(uint256(0xff));
  bytes32 private constant AxelarDataStorageLocation =
    0x425f6b37f295fd163c4560fd76ae5ecda01050f2bd962b6c5d8a382923214c00;

  /// @notice Initialize AxelarExecutable module
  /// @param gateway Address of Axelar gateway
  /// @param gasService address of Axelar gas service
  function __AxelarExecutable_init(address gateway, address gasService) internal onlyInitializing {
    require(gateway != address(0), InvalidAddress());
    require(gasService != address(0), InvalidAddress());

    AxelarData storage $ = _getAxelarData();
    $.gasService = gasService;
    $.gateway = gateway;
  }

  function _getAxelarData() internal pure returns (AxelarData storage $) {
    assembly {
      $.slot := AxelarDataStorageLocation
    }
  }

  ///@notice Get address of Axelar gateway
  function getAxelarGateway() external view returns (address) {
    AxelarData storage $ = _getAxelarData();
    return $.gateway;
  }

  ///@notice Get address of Axelar gas service
  function getAxelarGasService() external view returns (address) {
    AxelarData storage $ = _getAxelarData();
    return $.gasService;
  }

  ///@notice Execute command with payload
  ///@dev Should be called by Axelar relayers
  function execute(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes calldata payload
  ) external {
    AxelarData storage $ = _getAxelarData();
    if (!IAxelarGateway($.gateway).validateContractCall(commandId, sourceChain, sourceAddress, keccak256(payload)))
      revert NotApprovedByGateway();

    //TODO: validate sourceChain and sourceAddress

    _handleExecute(payload);
  }

  ///@notice Execute command with payload and tokens amount
  ///@dev Should be called by Axelar relayers
  function executeWithToken(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes calldata payload,
    string calldata tokenSymbol,
    uint256 amount
  ) external {
    AxelarData storage $ = _getAxelarData();
    if (
      !IAxelarGateway($.gateway).validateContractCallAndMint(
        commandId,
        sourceChain,
        sourceAddress,
        keccak256(payload),
        tokenSymbol,
        amount
      )
    ) revert NotApprovedByGateway();

    _handleExecuteWithToken(payload, tokenSymbol, amount);
  }

  function _handleExecute(bytes calldata payload) internal virtual {}

  function _handleExecuteWithToken(
    bytes calldata payload,
    string calldata tokenSymbol,
    uint256 amount
  ) internal virtual {}
}
