use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::state::{QueueParams, StakeItem, StakeStatsItem, UnstakeItem};
use crate::tests::utils::{
    create_stake_response_payload, get_stake_queue_item, get_stake_queue_params, get_stake_stats,
    get_unstake_queue_item, get_unstake_queue_params, instantiate_contract, stake_and_unstake,
};
use crate::types::{StakeActionStage, StakeResponseData, Status, UnstakeActionStage};
use cosmwasm_std::testing::message_info;
use cosmwasm_std::{coins, Coin, Uint128, Uint256};

#[test]
fn test_init_stake_one_coin() {
    let mut ctx = instantiate_contract();
    let stake_amount = Uint128::from(1000_u32);
    let (token_denom, _) = ctx.tokens.first().clone().unwrap();

    // init stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user.clone(),
            &coins(stake_amount.u128(), token_denom.clone()),
        ),
        ExecuteMsg::Stake,
    )
    .unwrap();

    // check states
    let stake_queue_params =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        stake_queue_params,
        QueueParams {
            pending_count: 1_u64,
            next_id: 2_u64
        }
    );

    let stake_queue_item =
        get_stake_queue_item(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone(), 1).unwrap();
    assert_eq!(
        stake_queue_item,
        StakeItem {
            user: ctx.user,
            token_amount: stake_amount,
            action_stage: StakeActionStage::WaitingExecution,
        }
    );

    let stake_stats = get_stake_stats(ctx.deps.as_ref(), ctx.env.clone(), token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::from(stake_amount),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    // todo: check events ?
}

#[test]
fn test_stake_in_two_tx() {
    let mut ctx = instantiate_contract();

    let (token_denom, _) = ctx.tokens.first().clone().unwrap();
    let stake_amount_1 = Uint128::from(1000_u32);
    let stake_amount_2 = Uint128::from(2000_u32);
    let staked_total = stake_amount_1 + stake_amount_2;

    // init first stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user.clone(),
            &vec![Coin {
                denom: token_denom.clone(),
                amount: stake_amount_1,
            }],
        ),
        ExecuteMsg::Stake,
    )
    .unwrap();

    // init second stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user.clone(),
            &vec![Coin {
                denom: token_denom.clone(),
                amount: stake_amount_2,
            }],
        ),
        ExecuteMsg::Stake,
    )
    .unwrap();

    // check stats
    let stake_stats = get_stake_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::from(staked_total),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    // check queue states
    let stake_queue_params =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        stake_queue_params,
        QueueParams {
            pending_count: 2_u64,
            next_id: 3_u64,
        }
    );

    let stake_queue_item_1 =
        get_stake_queue_item(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone(), 1).unwrap();
    assert_eq!(
        stake_queue_item_1,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount_1,
            action_stage: StakeActionStage::WaitingExecution,
        }
    );

    let stake_queue_item_2 =
        get_stake_queue_item(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone(), 2).unwrap();
    assert_eq!(
        stake_queue_item_2,
        StakeItem {
            user: ctx.user,
            token_amount: stake_amount_2,
            action_stage: StakeActionStage::WaitingExecution,
        }
    );

    // todo: check events?
}

#[test]
fn test_stake_response_successful() {
    let mut ctx = instantiate_contract();
    let (token_denom, token_config) = ctx.tokens.first().clone().unwrap();
    let stake_amount = Uint128::from(1000_u32);
    let lp_token_amount = stake_amount + Uint128::one();

    // init stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user.clone(),
            &coins(stake_amount.u128(), token_denom.clone()),
        ),
        ExecuteMsg::Stake,
    )
    .unwrap();

    let stake_id = 1_u64;
    let reinit_unstake_id = 0;
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

    // check stats
    let stake_stats = get_stake_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    // check queue states
    let stake_queue_params =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        stake_queue_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let stake_queue_item_after =
        get_stake_queue_item(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone(), 1).unwrap();
    assert_eq!(
        stake_queue_item_after,
        StakeItem {
            user: ctx.user,
            token_amount: stake_amount,
            action_stage: StakeActionStage::Executed,
        }
    );

    // todo: check LP tokens are minted
}

#[test]
fn test_stake_response_successful_with_reinit() {
    let mut ctx = instantiate_contract();
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();
    let stake_amount = Uint128::from(1000_u32);
    let lp_token_amount = stake_amount + Uint128::one();

    let unstake_user = ctx.unstake_user.clone();
    let unstake_details = stake_and_unstake(&mut ctx, &unstake_user, &token_denom, &token_config);

    // init stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user.clone(),
            &coins(stake_amount.u128(), token_denom.clone()),
        ),
        ExecuteMsg::Stake,
    )
    .unwrap();

    let stake_id = 2_u64;
    let response_payload = create_stake_response_payload(StakeResponseData {
        status: Status::Success,
        stake_id,
        reinit_unstake_id: unstake_details.unstake_id,
        lp_token_amount,
    });

    // response for stake action
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
            lp_token_amount: Uint256::from(lp_token_amount),
            pending_unstake_lp_token_amount: Uint256::zero(),
        }
    );

    // check stake queue states
    let stake_queue_params =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        stake_queue_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 3_u64,
        }
    );

    let stake_queue_item_after =
        get_stake_queue_item(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone(), 2).unwrap();
    assert_eq!(
        stake_queue_item_after,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::Executed,
        }
    );

    // check unstake queue states
    let unstake_queue_params =
        get_unstake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        unstake_queue_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let unstake_queue_item_after = get_unstake_queue_item(
        ctx.deps.as_ref(),
        ctx.env.clone(),
        token_denom.clone(),
        unstake_details.unstake_id,
    )
    .unwrap();
    assert_eq!(
        unstake_queue_item_after,
        UnstakeItem {
            user: ctx.unstake_user.clone(),
            lp_token_amount: unstake_details.lp_token_amount,
            action_stage: UnstakeActionStage::Executed,
        }
    );

    // todo: check unstake account received funds
    // todo: check LP tokens are minted
}

#[test]
fn test_stake_response_fail() {
    let mut ctx = instantiate_contract();
    let stake_amount = Uint128::from(1000_u128);
    let (token_denom, token_config) = ctx.tokens.first().clone().unwrap();

    // init stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user.clone(),
            &coins(stake_amount.u128(), token_denom.clone()),
        ),
        ExecuteMsg::Stake,
    )
    .unwrap();

    // response for stake action
    let stake_id = 1_u64;
    let reinit_unstake_id = 0;
    let lp_token_amount = Uint128::zero();
    let response_payload = create_stake_response_payload(StakeResponseData {
        status: Status::Fail,
        stake_id,
        reinit_unstake_id,
        lp_token_amount,
    });

    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.axelar.clone(),
            &coins(stake_amount.u128(), token_denom.clone()),
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

    // check queue states
    let stake_queue_params =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        stake_queue_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let stake_queue_item_after =
        get_stake_queue_item(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone(), 1).unwrap();
    assert_eq!(
        stake_queue_item_after,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::Failed,
        }
    );

    // todo: check LP tokens are not minted
    // todo: check deposited tokens are returned to user
}

#[test]
fn test_stake_response_fail_with_reinit() {
    let mut ctx = instantiate_contract();
    let stake_amount = Uint128::from(1000_u128);
    let (token_denom, token_config) = ctx.tokens.first().unwrap().clone();
    let unstake_user = ctx.unstake_user.clone();
    let unstake_details = stake_and_unstake(&mut ctx, &unstake_user, &token_denom, &token_config);

    // init stake
    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.user.clone(),
            &coins(stake_amount.u128(), token_denom.clone()),
        ),
        ExecuteMsg::Stake,
    )
    .unwrap();

    // response for stake action
    let stake_id = 2_u64;
    let lp_token_amount = Uint128::zero();
    let response_payload = create_stake_response_payload(StakeResponseData {
        status: Status::Fail,
        stake_id,
        reinit_unstake_id: unstake_details.unstake_id,
        lp_token_amount,
    });

    execute(
        ctx.deps.as_mut(),
        ctx.env.clone(),
        message_info(
            &ctx.axelar.clone(),
            &coins(stake_amount.u128(), token_denom.clone()),
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

    // check queue states
    let stake_queue_params =
        get_stake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        stake_queue_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 3_u64,
        }
    );

    let stake_queue_item_after =
        get_stake_queue_item(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone(), 2).unwrap();
    assert_eq!(
        stake_queue_item_after,
        StakeItem {
            user: ctx.user.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::Failed,
        }
    );

    // check unstake queue states
    let unstake_queue_params =
        get_unstake_queue_params(ctx.deps.as_ref(), ctx.env.clone(), token_denom.clone());
    assert_eq!(
        unstake_queue_params,
        QueueParams {
            pending_count: 0_u64,
            next_id: 2_u64,
        }
    );

    let unstake_queue_item_after = get_unstake_queue_item(
        ctx.deps.as_ref(),
        ctx.env.clone(),
        token_denom.clone(),
        unstake_details.unstake_id,
    )
    .unwrap();
    assert_eq!(
        unstake_queue_item_after,
        UnstakeItem {
            user: ctx.unstake_user.clone(),
            lp_token_amount: unstake_details.lp_token_amount,
            action_stage: UnstakeActionStage::Executed,
        }
    );

    // todo: check LP tokens are not minted
    // todo: check deposited tokens are returned to user
    // todo: check unstake account received funds
}
