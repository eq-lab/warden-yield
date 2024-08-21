#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{
    try_add_token, try_disallow_mint, try_handle_response, try_init_stake, try_init_unstake,
    try_mint_lp_token, try_reinit, try_update_token_config,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{
    query_contract_config, query_stake_item, query_stake_params, query_stake_stats,
    query_tokens_configs, query_unstake_item, query_unstake_params,
};
use crate::reply::handle_lp_token_mint_reply;
use crate::state::{
    ContractConfigState, QueueParams, StakeStatsItem, CONTRACT_CONFIG, STAKE_PARAMS, STAKE_STATS,
    TOKEN_CONFIG, TOKEN_DENOM_BY_SOURCE, UNSTAKE_PARAMS,
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
        lp_token_code_id: msg.lp_token_code_id,
        is_mint_allowed: true,
    };
    CONTRACT_CONFIG.save(deps.storage, &contract_config)?;

    for (token_denom, config) in &msg.tokens {
        TOKEN_CONFIG.save(deps.storage, token_denom, config)?;
        STAKE_STATS.save(deps.storage, token_denom, &StakeStatsItem::default())?;
        STAKE_PARAMS.save(
            deps.storage,
            token_denom,
            &QueueParams {
                pending_count: 0,
                next_id: 1,
            },
        )?;
        UNSTAKE_PARAMS.save(
            deps.storage,
            token_denom,
            &QueueParams {
                pending_count: 0,
                next_id: 1,
            },
        )?;

        TOKEN_DENOM_BY_SOURCE.save(
            deps.storage,
            (&config.chain, &config.evm_yield_contract),
            token_denom,
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
        ExecuteMsg::Reinit { token_denom } => try_reinit(deps, env, info, token_denom),
        ExecuteMsg::MintLpToken {
            recipient,
            lp_token_address,
            amount,
        } => try_mint_lp_token(deps, env, info, recipient, lp_token_address, amount),
        ExecuteMsg::AddToken {
            token_denom,
            is_stake_enabled,
            is_unstake_enabled,
            chain,
            symbol,
            name,
            evm_yield_contract,
            evm_address,
            lp_token_denom,
        } => try_add_token(
            deps,
            env,
            info,
            token_denom,
            is_stake_enabled,
            is_unstake_enabled,
            chain,
            symbol,
            name,
            evm_yield_contract,
            evm_address,
            lp_token_denom,
        ),
        ExecuteMsg::UpdateTokenConfig {
            token_denom,
            config,
        } => try_update_token_config(deps, env, info, token_denom, config),
        ExecuteMsg::HandleResponse {
            source_chain,
            source_address,
            payload,
        } => try_handle_response(deps, env, info, source_chain, source_address, payload),
        ExecuteMsg::DisallowMint => try_disallow_mint(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractConfig => to_json_binary(&query_contract_config(deps)?),
        QueryMsg::TokensConfigs => to_json_binary(&query_tokens_configs(deps)?),
        QueryMsg::StakeStats => to_json_binary(&query_stake_stats(deps)?),
        QueryMsg::StakeParams { token_denom } => {
            to_json_binary(&query_stake_params(deps, token_denom)?)
        }
        QueryMsg::UnstakeParams { token_denom } => {
            to_json_binary(&query_unstake_params(deps, token_denom)?)
        }
        QueryMsg::StakeElem { token_denom, id } => {
            to_json_binary(&query_stake_item(deps, token_denom, id)?)
        }
        QueryMsg::UnstakeElem { token_denom, id } => {
            to_json_binary(&query_unstake_item(deps, token_denom, id)?)
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
