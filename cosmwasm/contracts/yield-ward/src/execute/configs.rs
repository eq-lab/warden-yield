use crate::helpers::assert_msg_sender_is_admin;
use crate::state::{ContractConfigState, CONTRACT_CONFIG, TOKEN_CONFIG};
use crate::types::TokenConfig;
use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};

pub fn try_update_token_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: String,
    config: TokenConfig,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    if !TOKEN_CONFIG.has(deps.storage, &token_denom) {
        return Err(ContractError::UnknownToken(token_denom.clone()));
    }
    TOKEN_CONFIG.save(deps.storage, &token_denom, &config)?;

    Ok(Response::new().add_event(Event::new("update_token_config")))
}

pub fn try_update_contract_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_config: ContractConfigState,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    CONTRACT_CONFIG.save(deps.storage, &contract_config)?;

    Ok(Response::default())
}
