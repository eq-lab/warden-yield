#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{
    try_add_token, try_handle_response, try_init_stake, try_init_unstake, try_update_token_config,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{
    query_contract_config, query_stake_queue_item, query_stake_queue_params, query_tokens_configs,
    query_tokens_stats, query_unstake_queue_item, query_unstake_queue_params,
};
use crate::reply::handle_lp_token_mint_reply;
use crate::state::{
    ContractConfigState, QueueParams, TokenStats, CONTRACT_CONFIG_STATE, STAKE_QUEUE_PARAMS,
    TOKENS_CONFIGS_STATE, TOKENS_STATS_STATE, UNSTAKE_QUEUE_PARAMS,
};
use crate::types::ReplyType;

// version info for migration info
const CONTRACT_NAME: &str = "warden-yield";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Use CW2 to set the contract version, this is needed for migrations
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let contract_config = ContractConfigState {
        owner: info.sender.clone(),
        axelar: msg.axelar,
    };
    CONTRACT_CONFIG_STATE.save(deps.storage, &contract_config)?;

    for (token_denom, config) in &msg.tokens {
        TOKENS_CONFIGS_STATE.save(deps.storage, token_denom.clone(), config)?;
        TOKENS_STATS_STATE.save(deps.storage, token_denom.clone(), &TokenStats::default())?;
        STAKE_QUEUE_PARAMS.save(
            deps.storage,
            token_denom.clone(),
            &QueueParams {
                count_active: 0,
                end: 1,
            },
        )?;
        UNSTAKE_QUEUE_PARAMS.save(
            deps.storage,
            token_denom.clone(),
            &QueueParams {
                count_active: 0,
                end: 1,
            },
        )?;
    }

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Stake => try_init_stake(deps, env, info),
        ExecuteMsg::Unstake => try_init_unstake(deps, env, info),
        ExecuteMsg::AddToken {
            token_denom,
            config,
        } => try_add_token(deps, env, info, token_denom, config),
        ExecuteMsg::UpdateTokenConfig {
            token_denom,
            config,
        } => try_update_token_config(deps, env, info, token_denom, config),
        ExecuteMsg::HandleResponse {
            source_chain,
            source_address,
            payload,
        } => try_handle_response(deps, env, info, source_chain, source_address, payload),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractConfig => to_json_binary(&query_contract_config(deps)?),
        QueryMsg::TokensConfigs => to_json_binary(&query_tokens_configs(deps)?),
        QueryMsg::TokensStats => to_json_binary(&query_tokens_stats(deps)?),
        QueryMsg::StakeQueueParams { token_denom } => {
            to_json_binary(&query_stake_queue_params(deps, token_denom)?)
        }
        QueryMsg::UnstakeQueueParams { token_denom } => {
            to_json_binary(&query_unstake_queue_params(deps, token_denom)?)
        }
        QueryMsg::StakeQueueElem { token_denom, id } => {
            to_json_binary(&query_stake_queue_item(deps, token_denom, id)?)
        }
        QueryMsg::UnstakeQueueElem { token_denom, id } => {
            to_json_binary(&query_unstake_queue_item(deps, token_denom, id)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match ReplyType::try_from(&msg.id) {
        Ok(ReplyType::LpMint) => handle_lp_token_mint_reply(deps, env, msg),
        _ => Err(ContractError::UnrecognizedReply(msg.id)),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // No state migrations performed, just returned a Response
    Ok(Response::default())
}
