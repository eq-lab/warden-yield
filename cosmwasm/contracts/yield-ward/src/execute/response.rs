use crate::encoding::decode_payload_action_type;
use crate::execute::reinit::try_handle_reinit_response;
use crate::execute::response_stake::try_handle_stake_response;
use crate::execute::response_unstake::try_handle_unstake_response;
use crate::helpers::assert_msg_sender_is_axelar;
use crate::types::ActionType;
use crate::ContractError;
use cosmwasm_std::{Addr, Binary, DepsMut, Env, Response, Uint128};

pub fn try_handle_response(
    deps: DepsMut,
    env: Env,
    // info: MessageInfo,
    sender: Addr,
    source_chain: String,
    source_address: String,
    payload: Binary,
    token_amount: Uint128,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_axelar(deps.as_ref(), &sender)?;

    let action_type =
        decode_payload_action_type(&payload).ok_or(ContractError::InvalidActionType)?;

    // skip ActionId first byte
    let payload = &payload[1..];
    match action_type {
        ActionType::Stake => try_handle_stake_response(
            deps,
            env,
            source_chain,
            source_address,
            payload,
            token_amount,
        ),
        ActionType::Unstake => try_handle_unstake_response(
            deps,
            env,
            source_chain,
            source_address,
            payload,
            token_amount,
        ),
        ActionType::Reinit => try_handle_reinit_response(
            deps,
            env,
            source_chain,
            source_address,
            payload,
            token_amount,
        ),
    }
}
