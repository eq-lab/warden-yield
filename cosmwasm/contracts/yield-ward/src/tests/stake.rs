use crate::state::{QueueParams, StakeItem, StakeStatsItem, UnstakeItem};
use crate::tests::utils::call::{call_stake, call_stake_and_unstake, call_stake_response};
use crate::tests::utils::init::instantiate_yield_ward_contract_with_tokens;
use crate::tests::utils::query::{
    get_stake_item, get_stake_params, get_stake_stats, get_unstake_item, get_unstake_params,
};
use crate::types::{StakeActionStage, Status, UnstakeActionStage};
use cosmwasm_std::{Uint128, Uint256};
#[test]
fn test_init_stake_one_coin() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let stake_amount = Uint128::from(1000_u32);
    let fee_amount = Uint128::from(100_u32);
    let token_info = ctx.tokens.first().unwrap();

    // init stake
    call_stake(
        &mut app,
        &ctx,
        &ctx.user,
        token_info,
        stake_amount,
        fee_amount,
    );

    // check states
    let stake_params = get_stake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_params,
        QueueParams {
            pending_count: 1_u64,
            next_id: 2_u64
        }
    );

    let stake_item = get_stake_item(&app, &ctx, &token_info.deposit_token_denom, 1).unwrap();
    assert_eq!(
        stake_item,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::WaitingExecution,
            lp_token_amount: None
        }
    );

    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::from(stake_amount),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );
}

#[test]
fn test_stake_in_two_tx() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let token_info = ctx.tokens.first().unwrap();
    let stake_amount_1 = Uint128::from(1000_u32);
    let fee_amount_1 = Uint128::from(100_u32);
    let stake_amount_2 = Uint128::from(2000_u32);
    let fee_amount_2 = Uint128::from(200_u32);
    let staked_total = stake_amount_1 + stake_amount_2;

    // init first stake
    call_stake(
        &mut app,
        &ctx,
        &ctx.user,
        token_info,
        stake_amount_1,
        fee_amount_1,
    );

    // init second stake
    call_stake(
        &mut app,
        &ctx,
        &ctx.user,
        token_info,
        stake_amount_2,
        fee_amount_2,
    );

    // check stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::from(staked_total),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    // check queue states
    let stake_params = get_stake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_params,
        QueueParams {
            pending_count: 2_u64,
            next_id: 3_u64,
        }
    );

    let stake_item_1 = get_stake_item(&app, &ctx, &token_info.deposit_token_denom, 1).unwrap();
    assert_eq!(
        stake_item_1,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount_1,
            action_stage: StakeActionStage::WaitingExecution,
            lp_token_amount: None
        }
    );

    let stake_item_2 = get_stake_item(&app, &ctx, &token_info.deposit_token_denom, 2).unwrap();
    assert_eq!(
        stake_item_2,
        StakeItem {
            user: ctx.user,
            token_amount: stake_amount_2,
            action_stage: StakeActionStage::WaitingExecution,
            lp_token_amount: None
        }
    );
}

#[test]
fn test_stake_response_successful() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let token_info = ctx.tokens.first().unwrap();
    let stake_amount = Uint128::from(1000_u32);
    let fee_amount = Uint128::from(100_u32);
    let lp_token_amount = stake_amount + Uint128::one();

    // init stake
    call_stake(
        &mut app,
        &ctx,
        &ctx.user,
        token_info,
        stake_amount,
        fee_amount,
    );

    let stake_id = 1_u64;
    let reinit_unstake_id = 0;

    // response for stake action
    call_stake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Success,
        stake_id,
        reinit_unstake_id,
        Uint128::zero(),
        lp_token_amount,
    );

    // check stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    // check stake states
    let stake_params = get_stake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let stake_item_after = get_stake_item(&app, &ctx, &token_info.deposit_token_denom, 1).unwrap();
    assert_eq!(
        stake_item_after,
        StakeItem {
            user: ctx.user,
            token_amount: stake_amount,
            action_stage: StakeActionStage::Executed,
            lp_token_amount: Some(lp_token_amount)
        }
    );
}

#[test]
fn test_stake_response_successful_with_reinit() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let token_info = ctx.tokens.first().unwrap();
    let stake_amount = Uint128::from(1000_u32);
    let fee_amount = Uint128::from(100_u32);
    let lp_token_amount = stake_amount + Uint128::one();

    let unstake_user = ctx.unstake_user.clone();
    let unstake_details = call_stake_and_unstake(&mut app, &ctx, &unstake_user, &token_info);

    // init stake
    call_stake(
        &mut app,
        &ctx,
        &ctx.user,
        token_info,
        stake_amount,
        fee_amount,
    );
    let stake_id = 2_u64;

    // response for stake action
    call_stake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Success,
        stake_id,
        unstake_details.unstake_id,
        unstake_details.unstake_token_amount,
        lp_token_amount,
    );

    // check stats
    let stake_stats = get_stake_stats(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    // check stake queue states
    let stake_params = get_stake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 3_u64,
        }
    );

    let stake_item_after = get_stake_item(&app, &ctx, &token_info.deposit_token_denom, 2).unwrap();
    assert_eq!(
        stake_item_after,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::Executed,
            lp_token_amount: Some(lp_token_amount)
        }
    );

    // check unstake queue states
    let unstake_params = get_unstake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        unstake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let unstake_item_after = get_unstake_item(
        &app,
        &ctx,
        &token_info.deposit_token_denom,
        unstake_details.unstake_id,
    )
    .unwrap();
    assert_eq!(
        unstake_item_after,
        UnstakeItem {
            user: ctx.unstake_user.clone(),
            lp_token_amount: unstake_details.lp_token_amount,
            action_stage: UnstakeActionStage::Executed,
            token_amount: Some(unstake_details.unstake_token_amount)
        }
    );
}

#[test]
fn test_stake_response_fail() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let stake_amount = Uint128::from(1000_u128);
    let fee_amount = Uint128::from(100_u32);
    let token_info = ctx.tokens.first().unwrap();

    // init stake
    call_stake(
        &mut app,
        &ctx,
        &ctx.user,
        token_info,
        stake_amount,
        fee_amount,
    );

    // response for stake action
    let stake_id = 1_u64;
    let reinit_unstake_id = 0;
    let lp_token_amount = Uint128::zero();

    call_stake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Fail,
        stake_id,
        reinit_unstake_id,
        Uint128::zero(),
        lp_token_amount,
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

    // check queue states
    let stake_params = get_stake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let stake_item_after = get_stake_item(&app, &ctx, &token_info.deposit_token_denom, 1).unwrap();
    assert_eq!(
        stake_item_after,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::Failed,
            lp_token_amount: None
        }
    );
}

#[test]
fn test_stake_response_fail_with_reinit() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let stake_amount = Uint128::from(1000_u128);
    let fee_amount = Uint128::from(100_u32);
    let token_info = ctx.tokens.get(0).unwrap();
    let unstake_user = ctx.unstake_user.clone();
    let unstake_details = call_stake_and_unstake(&mut app, &ctx, &unstake_user, &token_info);

    // init stake
    call_stake(
        &mut app,
        &ctx,
        &ctx.user,
        token_info,
        stake_amount,
        fee_amount,
    );

    // response for stake action
    let stake_id = 2_u64;
    let lp_token_amount = Uint128::zero();
    call_stake_response(
        &mut app,
        &ctx,
        token_info,
        Status::Fail,
        stake_id,
        unstake_details.unstake_id,
        unstake_details.unstake_token_amount,
        lp_token_amount,
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

    // check stake states
    let stake_params = get_stake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 3_u64,
        }
    );

    let stake_item_after = get_stake_item(&app, &ctx, &token_info.deposit_token_denom, 2).unwrap();
    assert_eq!(
        stake_item_after,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::Failed,
            lp_token_amount: None
        }
    );

    // check unstake states
    let unstake_params = get_unstake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        unstake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let unstake_item_after = get_unstake_item(
        &app,
        &ctx,
        &token_info.deposit_token_denom,
        unstake_details.unstake_id,
    )
    .unwrap();
    assert_eq!(
        unstake_item_after,
        UnstakeItem {
            user: ctx.unstake_user.clone(),
            lp_token_amount: unstake_details.lp_token_amount,
            action_stage: UnstakeActionStage::Executed,
            token_amount: Some(unstake_details.unstake_token_amount)
        }
    );
}
