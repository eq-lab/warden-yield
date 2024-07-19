use crate::msg::{
    GetContractConfigResponse, GetQueueParamsResponse, GetStakeQueueItemResponse,
    GetTokensConfigsResponse, GetTokensStatsResponse, GetUnstakeQueueItemResponse,
};
use crate::state::{
    CONTRACT_CONFIG_STATE, STAKE_QUEUE, STAKE_QUEUE_PARAMS, TOKENS_CONFIGS_STATE,
    TOKENS_STATS_STATE, UNSTAKE_QUEUE, UNSTAKE_QUEUE_PARAMS,
};
use crate::types::TokenDenom;
use cosmwasm_std::{Deps, Order, StdResult};

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

pub fn query_stake_queue_params(
    deps: Deps,
    token_denom: TokenDenom,
) -> StdResult<GetQueueParamsResponse> {
    Ok(GetQueueParamsResponse {
        params: STAKE_QUEUE_PARAMS.load(deps.storage, token_denom)?,
    })
}

pub fn query_unstake_queue_params(
    deps: Deps,
    token_denom: TokenDenom,
) -> StdResult<GetQueueParamsResponse> {
    Ok(GetQueueParamsResponse {
        params: UNSTAKE_QUEUE_PARAMS.load(deps.storage, token_denom)?,
    })
}

pub fn query_stake_queue_item(
    deps: Deps,
    token_denom: TokenDenom,
    id: u64,
) -> StdResult<GetStakeQueueItemResponse> {
    Ok(GetStakeQueueItemResponse {
        item: STAKE_QUEUE.load(deps.storage, (token_denom, id))?,
    })
}

pub fn query_unstake_queue_item(
    deps: Deps,
    token_denom: TokenDenom,
    id: u64,
) -> StdResult<GetUnstakeQueueItemResponse> {
    Ok(GetUnstakeQueueItemResponse {
        item: UNSTAKE_QUEUE.load(deps.storage, (token_denom, id))?,
    })
}
