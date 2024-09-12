use crate::state::{QueueParams, StakeItem, StakeStatsItem, UnstakeItem};
use crate::tests::utils::call::{
    call_stake, call_stake_and_unstake, call_stake_response, call_unstake, call_unstake_response,
};
use crate::tests::utils::init::instantiate_yield_ward_contract_with_tokens;
use crate::tests::utils::query::{
    get_stake_item, get_stake_params, get_stake_stats, get_unstake_item, get_unstake_params,
};
use crate::tests::utils::types::{TestInfo, TokenTestInfo};
use crate::types::{StakeActionStage, Status, UnstakeActionStage};
use cosmwasm_std::{Uint128, Uint256};
use cw_multi_test::BasicApp;

fn stake_and_response(
    app: &mut BasicApp,
    ctx: &TestInfo,
    stake_amount: Uint128,
    token_info: &TokenTestInfo,
) -> Uint128 {
    let stake_params_before = get_stake_params(app, ctx, &token_info.deposit_token_denom);
    let token_stats_before = get_stake_stats(app, ctx, &token_info.deposit_token_denom);
    let stake_id = stake_params_before.next_id;

    // init stake
    call_stake(app, ctx, &ctx.user, token_info, stake_amount);

    // response for stake action
    let reinit_unstake_id = 0_u64;
    let lp_token_amount = Uint128::from(1001_u128);
    call_stake_response(
        app,
        &ctx,
        token_info,
        Status::Success,
        stake_id,
        reinit_unstake_id,
        Uint128::zero(),
        lp_token_amount,
    );

    // check stake states
    let stake_params = get_stake_params(app, ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_params,
        QueueParams {
            pending_count: stake_params_before.pending_count,
            next_id: stake_params_before.next_id + 1
        }
    );
    let stake_item = get_stake_item(app, ctx, &token_info.deposit_token_denom, stake_id).unwrap();
    assert_eq!(
        stake_item,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::Executed,
            lp_token_amount: Some(lp_token_amount)
        }
    );

    // check token stats
    let token_stats = get_stake_stats(app, ctx, &token_info.deposit_token_denom);
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
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.first().unwrap();

    let stake_amount = Uint128::from(1000_u32);
    let lp_token_amount = stake_and_response(&mut app, &ctx, stake_amount, &token_info);

    // init unstake
    call_unstake(&mut app, &ctx, &ctx.user, token_info, lp_token_amount);

    // check unstake states
    let unstake_id = 1_u64;
    let unstake_params = get_unstake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        unstake_params,
        QueueParams {
            pending_count: 1_u64,
            next_id: 2_u64
        }
    );

    let unstake_item =
        get_unstake_item(&app, &ctx, &token_info.deposit_token_denom, unstake_id).unwrap();
    assert_eq!(
        unstake_item,
        UnstakeItem {
            user: ctx.user.clone(),
            lp_token_amount,
            action_stage: UnstakeActionStage::WaitingRegistration,
            token_amount: None
        }
    );

    // check stake stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::from(0_u64),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::from(lp_token_amount),
        }
    );
}

#[test]
fn test_unstake_response_successful() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.first().unwrap();

    let stake_amount = Uint128::from(1000_u32);
    let lp_token_amount = stake_and_response(&mut app, &ctx, stake_amount, &token_info);

    // init unstake
    let unstake_id = 1_u64;
    call_unstake(&mut app, &ctx, &ctx.user, token_info, lp_token_amount);

    // response for unstake action
    call_unstake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Success,
        unstake_id,
        0,
        Uint128::zero(),
    );

    // check stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::from(lp_token_amount),
        }
    );
}

#[test]
fn test_unstake_response_successful_instant_reinit() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.first().unwrap();

    let stake_amount = Uint128::from(1000_u32);
    let unstake_amount = stake_amount + Uint128::one();
    let lp_token_amount = stake_and_response(&mut app, &ctx, stake_amount, &token_info);

    // init unstake
    let unstake_id = 1_u64;
    call_unstake(&mut app, &ctx, &ctx.user, token_info, lp_token_amount);

    // response for unstake action
    call_unstake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Success,
        unstake_id,
        unstake_id,
        unstake_amount,
    );

    // check stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    let unstake_params = get_unstake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        unstake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64
        }
    );

    let unstake_item =
        get_unstake_item(&app, &ctx, &token_info.deposit_token_denom, unstake_id).unwrap();
    assert_eq!(
        unstake_item,
        UnstakeItem {
            user: ctx.user.clone(),
            action_stage: UnstakeActionStage::Executed,
            lp_token_amount,
            token_amount: Some(unstake_amount)
        }
    );
}

#[test]
fn test_unstake_response_successful_with_reinit() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.first().unwrap();

    let stake_amount = Uint128::from(1000_u32);
    let unstake_amount = stake_amount + Uint128::one();
    let unstake_user = ctx.unstake_user.clone();

    let unstake_details = call_stake_and_unstake(&mut app, &ctx, &unstake_user, &token_info);
    let lp_token_amount = stake_and_response(&mut app, &ctx, stake_amount, &token_info);

    // init unstake
    let unstake_id = unstake_details.unstake_id + 1;
    call_unstake(&mut app, &ctx, &ctx.user, token_info, lp_token_amount);

    // response for unstake action
    call_unstake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Success,
        unstake_id,
        unstake_id,
        unstake_amount,
    );

    // check stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::from(lp_token_amount),
        }
    );

    let unstake_params = get_unstake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        unstake_params,
        QueueParams {
            pending_count: 1_u64,
            next_id: 3_u64
        }
    );

    let unstake_item =
        get_unstake_item(&app, &ctx, &token_info.deposit_token_denom, unstake_id).unwrap();
    assert_eq!(
        unstake_item,
        UnstakeItem {
            user: ctx.user.clone(),
            action_stage: UnstakeActionStage::Executed,
            lp_token_amount,
            token_amount: Some(unstake_amount)
        }
    );
}

#[test]
fn test_unstake_response_fail() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.first().unwrap();

    let stake_amount = Uint128::from(1000_u32);
    let lp_token_amount = stake_and_response(&mut app, &ctx, stake_amount, &token_info);

    // init unstake
    let unstake_id = 1_u64;
    call_unstake(&mut app, &ctx, &ctx.user, token_info, lp_token_amount);

    // response for unstake action
    call_unstake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Fail,
        unstake_id,
        0,
        Uint128::zero(),
    );

    // check stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::zero()
        }
    );
}

#[test]
fn test_unstake_response_fail_with_reinit() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.first().unwrap();

    let stake_amount = Uint128::from(1000_u32);
    let unstake_user = ctx.unstake_user.clone();

    let unstake_details = call_stake_and_unstake(&mut app, &ctx, &unstake_user, &token_info);
    let lp_token_amount = stake_and_response(&mut app, &ctx, stake_amount, &token_info);

    // init unstake
    let unstake_id = unstake_details.unstake_id + 1;
    call_unstake(&mut app, &ctx, &ctx.user, token_info, lp_token_amount);

    // response for unstake action
    call_unstake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Fail,
        unstake_id,
        unstake_details.unstake_id,
        unstake_details.unstake_token_amount,
    );

    // check stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::zero()
        }
    );
}
