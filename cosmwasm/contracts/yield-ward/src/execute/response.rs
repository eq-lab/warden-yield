use crate::encoding::decode_payload_action_type;
use crate::execute::reinit::try_handle_reinit_response;
use crate::execute::response_stake::try_handle_stake_response;
use crate::execute::response_unstake::try_handle_unstake_response;
use crate::helpers::assert_msg_sender_is_axelar;
use crate::types::ActionType;
use crate::ContractError;
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};

pub fn try_handle_response(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: Binary,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_axelar(deps.as_ref(), &info.sender)?;

    let action_type =
        decode_payload_action_type(&payload).ok_or(ContractError::InvalidActionType)?;

    // skip ActionId first byte
    let payload = &payload[1..];
    match action_type {
        ActionType::Stake => {
            try_handle_stake_response(deps, env, info, source_chain, source_address, payload)
        }
        ActionType::Unstake => {
            try_handle_unstake_response(deps, env, info, source_chain, source_address, payload)
        }
        ActionType::Reinit => {
            try_handle_reinit_response(deps, env, info, source_chain, source_address, payload)
        }
    }
}
