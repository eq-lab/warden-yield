#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{
    try_add_token, try_handle_stake_response, try_handle_unstake_response, try_stake, try_unstake,
    try_update_token_config,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{
    query_contract_config, query_tokens_configs, query_tokens_stats, query_user_stats,
};
use crate::reply::handle_lp_token_mint_reply;
use crate::state::{
    ContractConfigState, TokenStats, CONTRACT_CONFIG_STATE, TOKENS_CONFIGS_STATE,
    TOKENS_STATS_STATE,
};

// version info for migration info
const CONTRACT_NAME: &str = "warden-yield";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LP_MINT_REPLY_ID: u64 = 1u64;

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
        TOKENS_STATS_STATE.save(
            deps.storage,
            token_denom.clone(),
            &TokenStats {
                pending_stake: Uint128::default(),
                staked_shares_amount: Uint128::default(),
                pending_shares_unstake: Uint128::default(),
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
        ExecuteMsg::Stake => try_stake(deps, env, info),
        ExecuteMsg::Unstake { token_denom } => try_unstake(deps, env, info, token_denom),
        ExecuteMsg::AddToken {
            token_denom,
            config,
        } => try_add_token(deps, env, info, token_denom, config),
        ExecuteMsg::UpdateTokenConfig {
            token_denom,
            config,
        } => try_update_token_config(deps, env, info, token_denom, config),
        ExecuteMsg::HandleStakeResponse {
            account,
            token_evm,
            token_amount,
            shares_amount,
            status,
        } => try_handle_stake_response(
            deps,
            env,
            info,
            account,
            token_evm,
            token_amount,
            shares_amount,
            status,
        ),
        ExecuteMsg::HandleUnstakeResponse {
            account,
            token_evm,
            token_amount,
            shares_amount,
            status,
        } => try_handle_unstake_response(
            deps,
            env,
            info,
            account,
            token_evm,
            token_amount,
            shares_amount,
            status,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractConfig => to_json_binary(&query_contract_config(deps)?),
        QueryMsg::TokensConfigs => to_json_binary(&query_tokens_configs(deps)?),
        QueryMsg::TokensStats => to_json_binary(&query_tokens_stats(deps)?),
        QueryMsg::UserStats {
            account,
            token_denom,
        } => to_json_binary(&query_user_stats(deps, account, token_denom)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        LP_MINT_REPLY_ID => handle_lp_token_mint_reply(deps, env, msg),
        _ => Err(ContractError::UnrecognisedReply(msg.id)),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // No state migrations performed, just returned a Response
    Ok(Response::default())
}
