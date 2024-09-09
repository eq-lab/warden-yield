use crate::helpers::assert_msg_sender_is_admin;
use crate::state::{ContractConfigState, CONTRACT_CONFIG, TOKEN_CONFIG, TOKEN_DENOM_BY_SOURCE};
use crate::types::TokenConfig;
use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Order, Response, StdResult};

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

    let tokens_denoms = TOKEN_DENOM_BY_SOURCE
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    let (source_chain_old, source_address_old) = tokens_denoms
        .iter()
        .find_map(
            |((source_chain, source_address), td)| match td == &token_denom {
                true => Some((source_chain.as_str(), source_address.as_str())),
                false => None,
            },
        )
        .ok_or(ContractError::UnknownToken(token_denom.clone()))?;

    if source_chain_old != config.chain || source_address_old != config.evm_yield_contract {
        TOKEN_DENOM_BY_SOURCE.remove(deps.storage, (source_chain_old, source_address_old));
        TOKEN_DENOM_BY_SOURCE.save(
            deps.storage,
            (config.chain.as_str(), config.evm_yield_contract.as_str()),
            &token_denom,
        )?;
    }

    Ok(Response::new().add_event(
        Event::new("update_token_config")
            .add_attribute("token_symbol", config.deposit_token_symbol),
    ))
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
