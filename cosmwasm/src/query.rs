use crate::msg::{
    GetContractConfigResponse, GetTokensConfigsResponse, GetTokensStatsResponse,
    GetUserStatsResponse,
};
use crate::state::{
    TokenStats, CONTRACT_CONFIG_STATE, TOKENS_CONFIGS_STATE, TOKENS_STATS_STATE, USERS_STATS_STATE,
};
use crate::types::TokenDenom;
use cosmwasm_std::{Addr, Deps, Order, StdResult, Uint128};

pub fn query_contract_config(deps: Deps) -> StdResult<GetContractConfigResponse> {
    let config = CONTRACT_CONFIG_STATE.load(deps.storage)?;
    Ok(GetContractConfigResponse { config })
}

pub fn query_tokens_configs(deps: Deps) -> StdResult<GetTokensConfigsResponse> {
    let tokens_configs: StdResult<Vec<_>> = TOKENS_CONFIGS_STATE
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    Ok(GetTokensConfigsResponse {
        tokens: tokens_configs?,
    })
}

pub fn query_tokens_stats(deps: Deps) -> StdResult<GetTokensStatsResponse> {
    let tokens_stats: StdResult<Vec<_>> = TOKENS_STATS_STATE
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    Ok(GetTokensStatsResponse {
        stats: tokens_stats?,
    })
}

pub fn query_user_stats(
    deps: Deps,
    account: Addr,
    token_denom: TokenDenom,
) -> StdResult<GetUserStatsResponse> {
    // todo: fix this token check
    let _ = TOKENS_CONFIGS_STATE.load(deps.storage, token_denom.clone())?;

    let stats = USERS_STATS_STATE.load(deps.storage, (account, token_denom));
    if stats.is_ok() {
        return Ok(GetUserStatsResponse {
            stats: stats.unwrap(),
        });
    }
    Ok(GetUserStatsResponse {
        stats: TokenStats {
            pending_stake: Uint128::zero(),
            staked_shares_amount: Uint128::zero(),
            pending_shares_unstake: Uint128::zero(),
        },
    })
}
