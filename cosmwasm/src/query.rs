use crate::msg::{
    GetContractConfigResponse, GetQueueParamsResponse, GetStakeItemResponse, GetStakeStatsResponse,
    GetTokensConfigsResponse, GetUnstakeItemResponse,
};
use crate::state::{
    CONTRACT_CONFIG, STAKES, STAKE_PARAMS, STAKE_STATS, TOKEN_CONFIG, UNSTAKES, UNSTAKE_PARAMS,
};
use crate::types::TokenDenom;
use cosmwasm_std::{Deps, Order, StdResult};

pub fn query_contract_config(deps: Deps) -> StdResult<GetContractConfigResponse> {
    let config = CONTRACT_CONFIG.load(deps.storage)?;
    Ok(GetContractConfigResponse { config })
}

pub fn query_tokens_configs(deps: Deps) -> StdResult<GetTokensConfigsResponse> {
    let tokens_configs: StdResult<Vec<_>> = TOKEN_CONFIG
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    Ok(GetTokensConfigsResponse {
        tokens: tokens_configs?,
    })
}

pub fn query_stake_stats(deps: Deps) -> StdResult<GetStakeStatsResponse> {
    let tokens_stats: StdResult<Vec<_>> = STAKE_STATS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    Ok(GetStakeStatsResponse {
        stats: tokens_stats?,
    })
}

pub fn query_stake_params(
    deps: Deps,
    token_denom: TokenDenom,
) -> StdResult<GetQueueParamsResponse> {
    Ok(GetQueueParamsResponse {
        params: STAKE_PARAMS.load(deps.storage, &token_denom)?,
    })
}

pub fn query_unstake_params(
    deps: Deps,
    token_denom: TokenDenom,
) -> StdResult<GetQueueParamsResponse> {
    Ok(GetQueueParamsResponse {
        params: UNSTAKE_PARAMS.load(deps.storage, &token_denom)?,
    })
}

pub fn query_stake_item(
    deps: Deps,
    token_denom: TokenDenom,
    id: u64,
) -> StdResult<GetStakeItemResponse> {
    Ok(GetStakeItemResponse {
        item: STAKES.load(deps.storage, (&token_denom, id))?,
    })
}

pub fn query_unstake_item(
    deps: Deps,
    token_denom: TokenDenom,
    id: u64,
) -> StdResult<GetUnstakeItemResponse> {
    Ok(GetUnstakeItemResponse {
        item: UNSTAKES.load(deps.storage, (&token_denom, id))?,
    })
}