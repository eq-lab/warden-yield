use crate::helpers::assert_msg_sender_is_admin;
use crate::state::{
    AxelarConfigState, ContractConfigState, AXELAR_CONFIG, CONTRACT_CONFIG, TOKEN_CONFIG,
    TOKEN_DENOM_BY_LPT_ADDRESS, TOKEN_DENOM_BY_SOURCE,
};
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

    let config_old = TOKEN_CONFIG.load(deps.storage, &token_denom)?;

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

    let config_evm_yield_contract = config.evm_yield_contract.to_lowercase();

    if source_chain_old != config.chain || source_address_old != config_evm_yield_contract {
        TOKEN_DENOM_BY_SOURCE.remove(deps.storage, (source_chain_old, source_address_old));
        TOKEN_DENOM_BY_SOURCE.save(
            deps.storage,
            (config.chain.as_str(), config_evm_yield_contract.as_str()),
            &token_denom,
        )?;
    }

    if config_old.lpt_address != config.lpt_address {
        TOKEN_DENOM_BY_LPT_ADDRESS.remove(deps.storage, &config_old.lpt_address);
        TOKEN_DENOM_BY_LPT_ADDRESS.save(deps.storage, &config.lpt_address, &token_denom)?;
    }

    Ok(Response::new()
        .add_event(Event::new("update_token_config").add_attribute("token_denom", token_denom)))
}

pub fn try_update_contract_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_config: ContractConfigState,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    CONTRACT_CONFIG.save(deps.storage, &contract_config)?;

    Ok(Response::default().add_event(Event::new("update_contract_config")))
}

pub fn try_update_axelar_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    axelar_config: AxelarConfigState,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    AXELAR_CONFIG.save(deps.storage, &axelar_config)?;

    Ok(Response::default().add_event(Event::new("update_axelar_config")))
}
