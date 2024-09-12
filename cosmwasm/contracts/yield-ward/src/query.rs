use crate::msg::{
    GetContractConfigResponse, GetQueueParamsResponse, GetStakeItemResponse, GetStakeStatsResponse,
    GetTokenDenomByLptAddressResponse, GetTokenDenomBySourceResponse, GetTokensConfigsResponse,
    GetUnstakeItemResponse,
};
use crate::state::{
    CONTRACT_CONFIG, STAKES, STAKE_PARAMS, STAKE_STATS, TOKEN_CONFIG, TOKEN_DENOM_BY_LPT_ADDRESS,
    TOKEN_DENOM_BY_SOURCE, UNSTAKES, UNSTAKE_PARAMS,
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

pub fn query_all_tokens_denoms_by_source(deps: Deps) -> StdResult<GetTokenDenomBySourceResponse> {
    let tokens_denoms: StdResult<Vec<_>> = TOKEN_DENOM_BY_SOURCE
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let tokens_denoms: Vec<_> = tokens_denoms?
        .into_iter()
        .map(|((source_chain, source_address), token_denom)| {
            (source_chain, source_address, token_denom)
        })
        .collect();

    Ok(GetTokenDenomBySourceResponse { tokens_denoms })
}

pub fn query_all_tokens_denoms_by_lpt_address(
    deps: Deps,
) -> StdResult<GetTokenDenomByLptAddressResponse> {
    let tokens_denoms: StdResult<Vec<_>> = TOKEN_DENOM_BY_LPT_ADDRESS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let tokens_denoms: Vec<_> = tokens_denoms?
        .into_iter()
        .map(|(lpt_address, token_denom)| (lpt_address, token_denom))
        .collect();

    Ok(GetTokenDenomByLptAddressResponse { tokens_denoms })
}
