use crate::msg::{
    GetContractConfigResponse, GetQueueParamsResponse, GetStakeItemResponse, GetStakeStatsResponse,
    GetTokenDenomByLptAddressResponse, GetTokenDenomBySourceResponse, GetTokensConfigsResponse,
    GetUnstakeItemResponse, QueryMsg,
};
use crate::state::{ContractConfigState, QueueParams, StakeItem, StakeStatsItem, UnstakeItem};
use crate::tests::utils::types::TestInfo;
use crate::types::{TokenConfig, TokenDenom};
use cosmwasm_std::BankQuery::Balance;
use cosmwasm_std::QueryRequest::Bank;
use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::BasicApp;
use lp_token::contract::QueryMsg as Cw20QueryMsg;
use std::collections::HashMap;

pub fn get_contract_config(app: &BasicApp, ctx: &TestInfo) -> ContractConfigState {
    let response: GetContractConfigResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::ContractConfig {},
        )
        .unwrap();

    response.config
}

pub fn get_stake_params(app: &BasicApp, ctx: &TestInfo, token_denom: &TokenDenom) -> QueueParams {
    let response: GetQueueParamsResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::StakeParams {
                token_denom: token_denom.clone(),
            },
        )
        .unwrap();

    response.params
}

pub fn get_unstake_params(app: &BasicApp, ctx: &TestInfo, token_denom: &TokenDenom) -> QueueParams {
    let response: GetQueueParamsResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::UnstakeParams {
                token_denom: token_denom.clone(),
            },
        )
        .unwrap();

    response.params
}

pub fn get_stake_item(
    app: &BasicApp,
    ctx: &TestInfo,
    token_denom: &TokenDenom,
    id: u64,
) -> Option<StakeItem> {
    let response: GetStakeItemResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::StakeElem {
                token_denom: token_denom.clone(),
                id,
            },
        )
        .unwrap();

    Some(response.item)
}

pub fn get_unstake_item(
    app: &BasicApp,
    ctx: &TestInfo,
    token_denom: &TokenDenom,
    id: u64,
) -> Option<UnstakeItem> {
    let response: GetUnstakeItemResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::UnstakeElem {
                token_denom: token_denom.clone(),
                id,
            },
        )
        .unwrap();

    Some(response.item)
}

pub fn get_all_stake_stats(app: &BasicApp, ctx: &TestInfo) -> HashMap<TokenDenom, StakeStatsItem> {
    let response: GetStakeStatsResponse = app
        .wrap()
        .query_wasm_smart(ctx.yield_ward_address.to_string(), &QueryMsg::StakeStats {})
        .unwrap();

    let stats: HashMap<_, _> = response.stats.into_iter().collect();
    stats
}

pub fn get_stake_stats(app: &BasicApp, ctx: &TestInfo, token_denom: &TokenDenom) -> StakeStatsItem {
    let stake_stats = get_all_stake_stats(app, ctx);

    stake_stats[token_denom].clone()
}

pub fn get_all_tokens_configs(app: &BasicApp, ctx: &TestInfo) -> HashMap<TokenDenom, TokenConfig> {
    let response: GetTokensConfigsResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::TokensConfigs {},
        )
        .unwrap();

    let configs: HashMap<_, _> = response.tokens.into_iter().collect();
    configs
}

pub fn get_token_denom_by_source(
    app: &BasicApp,
    ctx: &TestInfo,
) -> HashMap<(String, String), TokenDenom> {
    let response: GetTokenDenomBySourceResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::TokenDenomBySource {},
        )
        .unwrap();

    response
        .tokens_denoms
        .into_iter()
        .map(|(source_chain, source_address, token_denom)| {
            ((source_chain, source_address), token_denom)
        })
        .collect()
}

pub fn get_token_denom_by_lpt_address(app: &BasicApp, ctx: &TestInfo) -> HashMap<Addr, TokenDenom> {
    let response: GetTokenDenomByLptAddressResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::TokenDenomByLptAddress {},
        )
        .unwrap();

    response.tokens_denoms.into_iter().collect()
}

pub fn get_token_config(app: &BasicApp, ctx: &TestInfo, token_denom: &String) -> TokenConfig {
    let token_config = get_all_tokens_configs(app, ctx);

    token_config[token_denom].clone()
}

pub fn get_cw20_balance(app: &BasicApp, cw20_address: &Addr, account: &Addr) -> Uint128 {
    let response: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_address.to_string(),
            &Cw20QueryMsg::Balance {
                address: account.to_string(),
            },
        )
        .unwrap();

    response.balance
}

pub fn get_bank_token_balance(app: &BasicApp, token_denom: &TokenDenom, account: &Addr) -> Uint128 {
    let balance: cosmwasm_std::BalanceResponse = app
        .wrap()
        .query(&Bank(Balance {
            address: account.to_string(),
            denom: token_denom.to_string(),
        }))
        .unwrap();

    balance.amount.amount
}
