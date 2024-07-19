use crate::contract::{execute, instantiate, query};
use crate::msg::{
    ExecuteMsg, GetQueueParamsResponse, GetStakeQueueItemResponse, GetTokensStatsResponse,
    GetUnstakeQueueItemResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{QueueParams, StakeQueueItem, TokenStats, UnstakeQueueItem};
use crate::types::{
    ReinitResponseData, StakeResponseData, Status, TokenConfig, TokenDenom, UnstakeActionStage,
    UnstakeResponseData,
};
use cosmwasm_std::testing::{
    message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{coins, from_json, Addr, Binary, Deps, Env, OwnedDeps, Uint128, Uint256};
use std::collections::HashMap;

pub struct TestContext {
    pub deps: OwnedDeps<MockStorage, MockApi, MockQuerier>,
    pub env: Env,
    pub admin: Addr,
    pub user: Addr,
    pub unstake_user: Addr,
    pub axelar: Addr,
    pub tokens: Vec<(TokenDenom, TokenConfig)>,
}

pub fn instantiate_contract() -> TestContext {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let admin = Addr::unchecked("admin_acc");
    let user = Addr::unchecked("user_acc");
    let unstake_user = Addr::unchecked("unstake_user_acc");
    let axelar = Addr::unchecked("axelar_acc");
    let tokens = create_tokens_config();

    instantiate(
        deps.as_mut(),
        env.clone(),
        message_info(&admin, &[]),
        InstantiateMsg {
            tokens: tokens.clone(),
            axelar: axelar.clone(),
        },
    )
    .unwrap();

    TestContext {
        deps,
        env,
        admin,
        user,
        unstake_user,
        axelar,
        tokens,
    }
}

fn create_tokens_config() -> Vec<(TokenDenom, TokenConfig)> {
    vec![
        (
            "token_1".into(),
            TokenConfig {
                is_stake_enabled: true,
                is_unstake_enabled: false,
                symbol: "TOKEN1".to_string(),
                lp_token_denom: "TOKEN1_LP".to_string(),
                evm_address: "0x0000000000000000000000000000000000000001".to_string(),
                evm_yield_contract: "0x0000000000000000000000000000000000000011".to_string(),
                chain: "Ethereum".to_string(),
            },
        ),
        (
            "token_2".into(),
            TokenConfig {
                is_stake_enabled: true,
                is_unstake_enabled: false,
                symbol: "TOKEN2".to_string(),
                lp_token_denom: "TOKEN2_LP".to_string(),
                evm_address: "0x0000000000000000000000000000000000000002".to_string(),
                evm_yield_contract: "0x0000000000000000000000000000000000000022".to_string(),
                chain: "Ethereum".to_string(),
            },
        ),
    ]
}

pub fn get_stake_queue_params(deps: Deps, env: Env, token_denom: TokenDenom) -> QueueParams {
    let response: GetQueueParamsResponse = from_json(
        query(
            deps,
            env.clone(),
            QueryMsg::StakeQueueParams { token_denom },
        )
        .unwrap(),
    )
    .unwrap();

    response.params
}

pub fn get_unstake_queue_params(deps: Deps, env: Env, token_denom: TokenDenom) -> QueueParams {
    let response: GetQueueParamsResponse = from_json(
        query(
            deps,
            env.clone(),
            QueryMsg::UnstakeQueueParams { token_denom },
        )
        .unwrap(),
    )
    .unwrap();

    response.params
}

pub fn get_stake_queue_item(
    deps: Deps,
    env: Env,
    token_denom: TokenDenom,
    id: u64,
) -> Option<StakeQueueItem> {
    let response: GetStakeQueueItemResponse = from_json(
        query(
            deps,
            env.clone(),
            QueryMsg::StakeQueueElem { token_denom, id },
        )
        .ok()?,
    )
    .ok()?;

    Some(response.item)
}

pub fn get_unstake_queue_item(
    deps: Deps,
    env: Env,
    token_denom: TokenDenom,
    id: u64,
) -> Option<UnstakeQueueItem> {
    let response: GetUnstakeQueueItemResponse = from_json(
        query(
            deps,
            env.clone(),
            QueryMsg::UnstakeQueueElem { token_denom, id },
        )
        .ok()?,
    )
    .ok()?;

    Some(response.item)
}

pub fn create_stake_response_payload(stake_response_data: StakeResponseData) -> Binary {
    let status = match stake_response_data.status {
        Status::Success => 0_u8,
        Status::Fail => 1_u8,
    };

    let payload: Vec<u8> = vec![0_u8, status]
        .into_iter()
        .chain(stake_response_data.stake_id.to_be_bytes().into_iter())
        .chain(
            stake_response_data
                .reinit_unstake_id
                .to_be_bytes()
                .into_iter(),
        )
        .chain(
            stake_response_data
                .lp_token_amount
                .to_be_bytes()
                .into_iter(),
        )
        .map(|x| x)
        .collect();

    Binary::new(payload)
}

pub fn create_unstake_response_payload(unstake_response_data: UnstakeResponseData) -> Binary {
    let status = match unstake_response_data.status {
        Status::Success => 0_u8,
        Status::Fail => 1_u8,
    };

    let payload: Vec<u8> = vec![1_u8, status]
        .into_iter()
        .chain(unstake_response_data.unstake_id.to_be_bytes().into_iter())
        .chain(
            unstake_response_data
                .reinit_unstake_id
                .to_be_bytes()
                .into_iter(),
        )
        .map(|x| x)
        .collect();

    Binary::new(payload)
}

pub fn create_reinit_response_payload(reinit_response_data: ReinitResponseData) -> Binary {
    let payload: Vec<u8> = vec![2_u8]
        .into_iter()
        .chain(
            reinit_response_data
                .reinit_unstake_id
                .to_be_bytes()
                .into_iter(),
        )
        .map(|x| x)
        .collect();

    Binary::new(payload)
}

pub fn get_tokens_stats(deps: Deps, env: Env) -> HashMap<TokenDenom, TokenStats> {
    let tokens_stats: GetTokensStatsResponse =
        from_json(query(deps, env, QueryMsg::TokensStats).unwrap()).unwrap();

    let stats: HashMap<_, _> = tokens_stats.stats.into_iter().collect();
    stats
}

pub fn get_token_stats(deps: Deps, env: Env, token_denom: &TokenDenom) -> TokenStats {
    let token_stats = get_tokens_stats(deps, env);

    token_stats[token_denom].clone()
}

pub struct UnstakeDetails {
    pub _stake_id: u64,
    pub _stake_amount: Uint128,
    pub unstake_id: u64,
    pub lp_token_amount: Uint128,
    pub unstake_token_amount: Uint128,
}

pub fn stake_and_unstake(
    ctx: &mut TestContext,
    user: &Addr,
    token_denom: &TokenDenom,
    token_config: &TokenConfig,
) -> UnstakeDetails {
    let stake_id =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone()).end;

    let stake_amount = Uint128::from(14000_u128);

    // init stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(&user, &coins(stake_amount.u128(), token_denom.clone())),
        ExecuteMsg::Stake,
    )
    .unwrap();

    let reinit_unstake_id = 0;
    let lp_token_amount = Uint128::from(1001_u128);
    let response_payload = create_stake_response_payload(StakeResponseData {
        status: Status::Success,
        stake_id,
        reinit_unstake_id,
        lp_token_amount,
    });

    // response for stake action
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(&ctx.axelar.clone(), &vec![]),
        ExecuteMsg::HandleResponse {
            source_chain: token_config.chain.clone(),
            source_address: token_config.evm_yield_contract.clone(),
            payload: response_payload,
        },
    )
    .unwrap();

    let unstake_id =
        get_unstake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone()).end;

    // init unstake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &user,
            &coins(lp_token_amount.u128(), &token_config.lp_token_denom),
        ),
        ExecuteMsg::Unstake,
    )
    .unwrap();

    // response for unstake action
    let response_payload = create_unstake_response_payload(UnstakeResponseData {
        status: Status::Success,
        unstake_id,
        reinit_unstake_id: 0_u64,
    });

    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(&ctx.axelar.clone(), &vec![]),
        ExecuteMsg::HandleResponse {
            source_chain: token_config.chain.clone(),
            source_address: token_config.evm_yield_contract.clone(),
            payload: response_payload,
        },
    )
    .unwrap();

    let unstake_queue_params =
        get_unstake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        unstake_queue_params,
        QueueParams {
            count_active: 1_u64,
            end: 2_u64,
        }
    );

    let unstake_queue_item = get_unstake_queue_item(
        ctx.deps.as_ref(),
        ctx.env.clone(),
        token_denom.clone(),
        unstake_id,
    )
    .unwrap();
    assert_eq!(
        unstake_queue_item,
        UnstakeQueueItem {
            user: user.clone(),
            lp_token_amount,
            action_stage: UnstakeActionStage::Registered
        }
    );

    let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        token_stats,
        TokenStats {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::from(lp_token_amount),
        }
    );

    UnstakeDetails {
        _stake_id: stake_id,
        _stake_amount: stake_amount,
        unstake_id,
        lp_token_amount,
        unstake_token_amount: stake_amount + Uint128::from(100_u128),
    }
}
