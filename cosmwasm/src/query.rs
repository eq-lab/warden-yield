use crate::msg::{
    GetContractConfigResponse, GetTokensConfigsResponse, GetTokensStatsResponse,
    GetUserStatsResponse,
};
use crate::state::{
    TokenStats, CONTRACT_CONFIG_STATE, TOKENS_CONFIGS_STATE, TOKENS_STATS_STATE, USERS_STATS_STATE,
};
use crate::types::TokenDenom;
use cosmwasm_std::{Addr, Deps, Order, StdError, StdResult};

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

    match stats {
        Ok(token_stats) => Ok(GetUserStatsResponse { stats: token_stats }),
        Err(StdError::NotFound{ kind: _kind, backtrace: _backtrace }) => Ok(GetUserStatsResponse { stats:  TokenStats::default()}),
        Err(other_error) => Err(other_error), // if possible
    }
}
