#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::add_token::try_add_token;
use crate::execute::configs::{try_update_contract_config, try_update_token_config};
use crate::execute::mint_lpt::{try_disallow_mint, try_mint_lp_token};
use crate::execute::receive_cw20::try_receive_cw20;
use crate::execute::reinit::try_reinit;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{
    query_contract_config, query_stake_item, query_stake_params, query_stake_stats,
    query_tokens_configs, query_unstake_item, query_unstake_params,
};
use crate::reply::handle_lp_token_mint_reply;
use crate::state::{ContractConfigState, CONTRACT_CONFIG};
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
        ExecuteMsg::Stake => {
            unimplemented!() // todo: return after finish CW20 deposit tests: try_init_stake(deps, env, info),
        }
        ExecuteMsg::Receive(msg) => try_receive_cw20(deps, env, info, msg),
        ExecuteMsg::Reinit { token_denom } => try_reinit(deps, env, info, token_denom),
        ExecuteMsg::MintLpToken {
            recipient,
            lp_token_address,
            amount,
        } => try_mint_lp_token(deps, env, info, recipient, lp_token_address, amount),
        ExecuteMsg::AddToken {
            token_denom,
            cw20_address,
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
            cw20_address,
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
        ExecuteMsg::UpdateContractConfig { contract_config } => {
            try_update_contract_config(deps, env, info, contract_config)
        }
        // ExecuteMsg::HandleResponse {
        //     source_chain,
        //     source_address,
        //     payload,
        // } => try_handle_response(deps, env, info, source_chain, source_address, payload),
        ExecuteMsg::DisallowMint => try_disallow_mint(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractConfig {} => to_json_binary(&query_contract_config(deps)?),
        QueryMsg::TokensConfigs {} => to_json_binary(&query_tokens_configs(deps)?),
        QueryMsg::StakeStats {} => to_json_binary(&query_stake_stats(deps)?),
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
