use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::state::{QueueParams, StakeItem, StakeStatsItem, UnstakeItem};
use crate::tests::utils::{
    create_stake_response_payload, create_unstake_response_payload, get_stake_queue_item,
    get_stake_queue_params, get_token_stats, get_unstake_queue_item, get_unstake_queue_params,
    instantiate_contract, stake_and_unstake, TestContext,
};
use crate::types::{
    StakeActionStage, StakeResponseData, Status, TokenConfig, TokenDenom, UnstakeActionStage,
    UnstakeResponseData,
};
use cosmwasm_std::testing::message_info;
use cosmwasm_std::{coins, Uint128, Uint256};

fn stake_and_response(
    ctx: &mut TestContext,
    stake_amount: Uint128,
    token_denom: &TokenDenom,
    token_config: &TokenConfig,
) -> Uint128 {
    let stake_queue_params_before =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    let token_stats_before = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), token_denom);
    let stake_id = stake_queue_params_before.next_id.clone();

    // init stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(&ctx.user, &coins(stake_amount.u128(), token_denom.clone())),
        ExecuteMsg::Stake,
    )
    .unwrap();

    // response for stake action
    let reinit_unstake_id = 0_u64;
    let lp_token_amount = Uint128::from(1001_u128);
    let response_payload = create_stake_response_payload(StakeResponseData {
        status: Status::Success,
        stake_id,
        reinit_unstake_id,
        lp_token_amount,
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

    // check stake queue states
    let stake_queue_params =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        stake_queue_params,
        QueueParams {
            pending_count: stake_queue_params_before.pending_count,
            next_id: stake_queue_params_before.next_id + 1
        }
    );
    let stake_queue_item = get_stake_queue_item(
        ctx.deps.as_ref(),
        ctx.env.clone(),
        token_denom.clone(),
        stake_id.clone(),
    )
    .unwrap();
    assert_eq!(
        stake_queue_item,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::Executed,
        }
    );

    // check token stats
    let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), token_denom);
    assert_eq!(
        token_stats,
        StakeStatsItem {
            pending_stake: token_stats_before.pending_stake,
            lp_token_amount: token_stats_before.lp_token_amount + Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: token_stats_before.pending_unstake_lp_token_amount
        }
    );

    lp_token_amount
}

#[test]
fn test_init_unstake_one_coin() {
    let mut ctx = instantiate_contract();
    let stake_amount = Uint128::from(1000_u32);
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();

    let lp_token_amount = stake_and_response(&mut ctx, stake_amount, &token_denom, &token_config);

    // init unstake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user,
            &coins(lp_token_amount.u128(), token_config.lp_token_denom),
        ),
        ExecuteMsg::Unstake,
    )
    .unwrap();

    // check unstake queue states
    let unstake_id = 1_u64;
    let unstake_queue_params =
        get_unstake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        unstake_queue_params,
        QueueParams {
            pending_count: 1_u64,
            next_id: 2_u64
        }
    );

    let unstake_queue_item = get_unstake_queue_item(
        ctx.deps.as_ref(),
        ctx.env.clone(),
        token_denom.clone(),
        unstake_id.clone(),
    )
    .unwrap();
    assert_eq!(
        unstake_queue_item,
        UnstakeItem {
            user: ctx.user.clone(),
            lp_token_amount,
            action_stage: UnstakeActionStage::WaitingRegistration
        }
    );

    // check token stats
    let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        token_stats,
        StakeStatsItem {
            pending_stake: Uint256::from(0_u64),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::from(lp_token_amount),
        }
    );

    // todo: check events ?
}

#[test]
fn test_unstake_response_successful() {
    let mut ctx = instantiate_contract();
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();
    let stake_amount = Uint128::from(1000_u32);
    let lp_token_amount = stake_and_response(&mut ctx, stake_amount, &token_denom, &token_config);

    // init unstake
    let unstake_id = 1_u64;
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user,
            &coins(lp_token_amount.u128(), token_config.lp_token_denom.clone()),
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

    // check stats
    let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        token_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::from(lp_token_amount),
        }
    );

    // todo: check LP tokens are burned
}

#[test]
fn test_unstake_response_successful_instant_reinit() {
    let mut ctx = instantiate_contract();
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();
    let stake_amount = Uint128::from(1000_u32);
    let unstake_amount = stake_amount + Uint128::one();
    let lp_token_amount = stake_and_response(&mut ctx, stake_amount, &token_denom, &token_config);

    // init unstake
    let unstake_id = 1_u64;
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user,
            &coins(lp_token_amount.u128(), token_config.lp_token_denom.clone()),
        ),
        ExecuteMsg::Unstake,
    )
    .unwrap();

    // response for unstake action
    let response_payload = create_unstake_response_payload(UnstakeResponseData {
        status: Status::Success,
        unstake_id,
        reinit_unstake_id: unstake_id,
    });
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.axelar.clone(),
            &coins(unstake_amount.u128(), token_denom.clone()),
        ),
        ExecuteMsg::HandleResponse {
            source_chain: token_config.chain.clone(),
            source_address: token_config.evm_yield_contract.clone(),
            payload: response_payload,
        },
    )
    .unwrap();

    // check stats
    let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        token_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    let unstake_queue_params =
        get_unstake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        unstake_queue_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64
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
        UnstakeItem {
            user: ctx.user.clone(),
            action_stage: UnstakeActionStage::Executed,
            lp_token_amount,
        }
    );

    // todo: check LP tokens are burned
    // todo: check user received deposit + rewards tokens
}

#[test]
fn test_unstake_response_successful_with_reinit() {
    let mut ctx = instantiate_contract();
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();
    let stake_amount = Uint128::from(1000_u32);
    let unstake_amount = stake_amount + Uint128::one();
    let unstake_user = ctx.unstake_user.clone();

    let unstake_details = stake_and_unstake(&mut ctx, &unstake_user, &token_denom, &token_config);
    let lp_token_amount = stake_and_response(&mut ctx, stake_amount, &token_denom, &token_config);

    // init unstake
    let unstake_id = unstake_details.unstake_id + 1;
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user,
            &coins(lp_token_amount.u128(), token_config.lp_token_denom.clone()),
        ),
        ExecuteMsg::Unstake,
    )
    .unwrap();

    // response for unstake action
    let response_payload = create_unstake_response_payload(UnstakeResponseData {
        status: Status::Success,
        unstake_id,
        reinit_unstake_id: unstake_id,
    });
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.axelar.clone(),
            &coins(unstake_amount.u128(), token_denom.clone()),
        ),
        ExecuteMsg::HandleResponse {
            source_chain: token_config.chain.clone(),
            source_address: token_config.evm_yield_contract.clone(),
            payload: response_payload,
        },
    )
    .unwrap();

    // check stats
    let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        token_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::from(lp_token_amount),
        }
    );

    let unstake_queue_params =
        get_unstake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        unstake_queue_params,
        QueueParams {
            pending_count: 1_u64,
            next_id: 3_u64
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
        UnstakeItem {
            user: ctx.user.clone(),
            action_stage: UnstakeActionStage::Executed,
            lp_token_amount,
        }
    );

    // todo: check LP tokens are burned
    // todo: check user received deposit + rewards tokens
}

#[test]
fn test_unstake_response_fail() {
    let mut ctx = instantiate_contract();
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();
    let stake_amount = Uint128::from(1000_u32);
    let lp_token_amount = stake_and_response(&mut ctx, stake_amount, &token_denom, &token_config);

    // init unstake
    let unstake_id = 1_u64;
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user,
            &coins(lp_token_amount.u128(), token_config.lp_token_denom.clone()),
        ),
        ExecuteMsg::Unstake,
    )
    .unwrap();

    // response for unstake action
    let response_payload = create_unstake_response_payload(UnstakeResponseData {
        status: Status::Fail,
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

    // check stats
    let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        token_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::zero()
        }
    );

    // todo: check LP tokens are returned to user
}

#[test]
fn test_unstake_response_fail_with_reinit() {
    let mut ctx = instantiate_contract();
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();
    let stake_amount = Uint128::from(1000_u32);
    let unstake_user = ctx.unstake_user.clone();

    let unstake_details = stake_and_unstake(&mut ctx, &unstake_user, &token_denom, &token_config);
    let lp_token_amount = stake_and_response(&mut ctx, stake_amount, &token_denom, &token_config);

    // init unstake
    let unstake_id = unstake_details.unstake_id + 1;
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user,
            &coins(lp_token_amount.u128(), token_config.lp_token_denom.clone()),
        ),
        ExecuteMsg::Unstake,
    )
    .unwrap();

    // response for unstake action
    let response_payload = create_unstake_response_payload(UnstakeResponseData {
        status: Status::Fail,
        unstake_id,
        reinit_unstake_id: unstake_details.unstake_id,
    });
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.axelar.clone(),
            &coins(
                unstake_details.unstake_token_amount.u128(),
                token_denom.clone(),
            ),
        ),
        ExecuteMsg::HandleResponse {
            source_chain: token_config.chain.clone(),
            source_address: token_config.evm_yield_contract.clone(),
            payload: response_payload,
        },
    )
    .unwrap();

    // check stats
    let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        token_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::zero()
        }
    );

    // todo: check LP tokens are returned to user
}
