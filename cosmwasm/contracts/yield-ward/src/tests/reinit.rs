use crate::state::{QueueParams, StakeStatsItem, UnstakeItem};
use crate::tests::utils::call::{call_reinit_response, call_stake_and_unstake};
use crate::tests::utils::init::instantiate_yield_ward_contract_with_tokens;
use crate::tests::utils::query::{get_stake_stats, get_unstake_item, get_unstake_params};
use crate::types::UnstakeActionStage;
use cosmwasm_std::Uint256;

#[test]
fn test_reinit() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.get(0).unwrap();
    let unstake_user = &ctx.unstake_user;

    let unstake_details = call_stake_and_unstake(&mut app, &ctx, unstake_user, &token_info);

    // reinit response
    call_reinit_response(
        &mut app,
        &ctx,
        token_info,
        unstake_details.unstake_id,
        unstake_details.unstake_token_amount,
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

    // check unstake states
    let unstake_params = get_unstake_params(&app, &ctx, &token_info.deposit_token_denom);
    assert_eq!(
        unstake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let unstake_item = get_unstake_item(
        &app,
        &ctx,
        &token_info.deposit_token_denom,
        unstake_details.unstake_id,
    )
    .unwrap();
    assert_eq!(
        unstake_item,
        UnstakeItem {
            user: ctx.unstake_user.clone(),
            lp_token_amount: unstake_details.lp_token_amount,
            action_stage: UnstakeActionStage::Executed,
            token_amount: Some(unstake_details.unstake_token_amount)
        }
    );
}
