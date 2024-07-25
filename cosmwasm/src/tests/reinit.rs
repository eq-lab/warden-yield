use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::state::{QueueParams, StakeStatsItem, UnstakeItem};
use crate::tests::utils::{
    create_reinit_response_payload, get_stake_stats, get_unstake_item, get_unstake_params,
    instantiate_contract, stake_and_unstake,
};
use crate::types::{ReinitResponseData, UnstakeActionStage};
use cosmwasm_std::testing::message_info;
use cosmwasm_std::{coins, Uint256};

#[test]
fn test_reinit() {
    let mut ctx = instantiate_contract();
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();

    let unstake_user = ctx.unstake_user.clone();
    let unstake_details = stake_and_unstake(&mut ctx, &unstake_user, &token_denom, &token_config);

    let response_payload = create_reinit_response_payload(ReinitResponseData {
        reinit_unstake_id: unstake_details.unstake_id,
    });

    // reinit response
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
    let stake_stats = get_stake_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    // check unstake states
    let unstake_params =
        get_unstake_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        unstake_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let unstake_item = get_unstake_item(
        ctx.deps.as_ref(),
        ctx.env.clone(),
        token_denom.clone(),
        unstake_details.unstake_id,
    )
    .unwrap();
    assert_eq!(
        unstake_item,
        UnstakeItem {
            user: ctx.unstake_user.clone(),
            lp_token_amount: unstake_details.lp_token_amount,
            action_stage: UnstakeActionStage::Executed,
        }
    );
}
