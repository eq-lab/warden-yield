use crate::state::{QueueParams, StakeItem, StakeStatsItem, UnstakeItem};
use crate::tests::utils::call::{call_stake, call_stake_and_unstake, call_stake_response};
use crate::tests::utils::init::instantiate_yield_ward_contract_with_tokens;
use crate::tests::utils::query::{
    get_all_tokens_configs, get_stake_item, get_stake_params, get_stake_stats, get_unstake_item,
    get_unstake_params,
};
use crate::types::{StakeActionStage, Status, UnstakeActionStage};
use crate::ContractError;
use cosmwasm_std::{Uint128, Uint256};
use cw_multi_test::error::anyhow;
use cw_multi_test::Executor;

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

#[test]
fn test_stake_wrong_funds_length() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let stake_amount = Uint128::from(1000_u128);
    let fee_amount = Uint128::from(100_u32);
    let token_info = ctx.tokens.get(0).unwrap();

    let coin = cosmwasm_std::coin(
        stake_amount.into(),
        token_info.deposit_token_denom.to_string(),
    );
    let funds = vec![coin.clone(), coin];

    let stake_result = app.execute(
        ctx.unstake_user,
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::Stake { fee_amount })
                .unwrap(),
            funds,
        }),
    );

    match stake_result {
        Ok(_) => panic!("Stake passed with wrong funds length"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            "Custom Error: Init stake message must have one type of coins as funds"
        ),
    }
}

#[test]
fn test_stake_passed_fee_is_gt_total_amount() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let stake_amount = Uint128::from(1000_u128);
    let fee_amount = stake_amount.saturating_add(Uint128::one());
    let token_info = ctx.tokens.get(0).unwrap();

    let stake_result = app.execute(
        ctx.unstake_user,
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::Stake { fee_amount })
                .unwrap(),
            funds: cosmwasm_std::coins(
                stake_amount.into(),
                token_info.deposit_token_denom.to_string(),
            ),
        }),
    );

    match stake_result {
        Ok(_) => panic!("Stake passed with wrong funds length"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            "Custom Error: Fee amount should be less than attached amount"
        ),
    }
}

#[test]
fn test_stake_disabled() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let stake_amount = Uint128::from(1000_u128);
    let fee_amount = Uint128::from(100_u128);
    let token_info = ctx.tokens.get(0).unwrap();

    let token_configs = get_all_tokens_configs(&app, &ctx);

    let mut config = token_configs
        .get(token_info.deposit_token_denom.as_str())
        .unwrap()
        .clone();

    if config.is_unstake_enabled {
        config.is_stake_enabled = false;

        app.execute(
            ctx.admin,
            cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: ctx.yield_ward_address.to_string(),
                msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::UpdateTokenConfig {
                    token_denom: token_info.deposit_token_denom.clone(),
                    config,
                })
                .unwrap(),
                funds: vec![],
            }),
        )
        .unwrap();
    }

    let stake_result = app.execute(
        ctx.unstake_user,
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::Stake { fee_amount })
                .unwrap(),
            funds: cosmwasm_std::coins(
                stake_amount.into(),
                token_info.deposit_token_denom.to_string(),
            ),
        }),
    );

    match stake_result {
        Ok(_) => panic!("Stake passed with wrong funds length"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            anyhow!(ContractError::StakeDisabled("LPT-zero".into()))
                .root_cause()
                .to_string()
        ),
    }
}

#[test]
fn test_wrong_stake_response() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let token_info = ctx.tokens.get(0).unwrap();

    let stake_amount = Uint128::from(1000_u32);
    let fee_amount = Uint128::from(100_u32);
    call_stake(
        &mut app,
        &ctx,
        &ctx.user,
        token_info,
        stake_amount,
        fee_amount,
    );

    let mut response_payload =
        super::utils::calldata::create_stake_response_payload(crate::types::StakeResponseData {
            status: Status::Success,
            stake_id: 1,
            reinit_unstake_id: 0,
            lp_token_amount: 1000_u64.into(),
        });

    let coin = cosmwasm_std::coin(1000_u64.into(), token_info.deposit_token_denom.to_string());

    let wrong_funds_len = app.execute(
        ctx.axelar.clone(),
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::HandleResponse {
                source_chain: token_info.chain.to_string(),
                source_address: token_info.evm_yield_contract.to_string(),
                payload: response_payload,
            })
            .unwrap(),
            funds: vec![coin.clone(), coin.clone()],
        }),
    );

    match wrong_funds_len {
        Ok(_) => panic!("stake response passed with wrong funds length"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            anyhow!(ContractError::CustomError(
                "Stake response has too much coins in message".into()
            ))
            .root_cause()
            .to_string()
        ),
    }

    response_payload =
        super::utils::calldata::create_stake_response_payload(crate::types::StakeResponseData {
            status: Status::Success,
            stake_id: 1,
            reinit_unstake_id: 1,
            lp_token_amount: 1000_u64.into(),
        });

    let reinit_unstake_id_not_zero = app.execute(
        ctx.axelar.clone(),
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::HandleResponse {
                source_chain: token_info.chain.to_string(),
                source_address: token_info.evm_yield_contract.to_string(),
                payload: response_payload,
            })
            .unwrap(),
            funds: vec![],
        }),
    );

    match reinit_unstake_id_not_zero {
        Ok(_) => panic!("stake response passed with zero funds length and non-zero reinit id"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            anyhow!(ContractError::CustomError(
                "Stake response: reinit_unstake_id != 0, but message have no tokens".into()
            ))
            .root_cause()
            .to_string()
        ),
    }

    response_payload =
        super::utils::calldata::create_stake_response_payload(crate::types::StakeResponseData {
            status: Status::Fail,
            stake_id: 1,
            reinit_unstake_id: 0,
            lp_token_amount: 1000_u64.into(),
        });

    let stake_fail_response = app.execute(
        ctx.axelar.clone(),
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::HandleResponse {
                source_chain: token_info.chain.to_string(),
                source_address: token_info.evm_yield_contract.to_string(),
                payload: response_payload,
            })
            .unwrap(),
            funds: vec![],
        }),
    );

    match stake_fail_response {
        Ok(_) => panic!("stake response passed with zero funds length and stake failed status"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            anyhow!(ContractError::CustomError(
                "Fail stake response must have tokens in message".into()
            ))
            .root_cause()
            .to_string()
        ),
    }

    response_payload =
        super::utils::calldata::create_stake_response_payload(crate::types::StakeResponseData {
            status: Status::Success,
            stake_id: 1,
            reinit_unstake_id: 0,
            lp_token_amount: 1000_u64.into(),
        });

    let stake_fail_response = app.execute(
        ctx.axelar.clone(),
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::HandleResponse {
                source_chain: token_info.chain.to_string(),
                source_address: token_info.evm_yield_contract.to_string(),
                payload: response_payload,
            })
            .unwrap(),
            funds: vec![coin.clone()],
        }),
    );

    match stake_fail_response {
        Ok(_) => panic!("stake response passed with singular funds length and both success and zero reinit id"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            anyhow!(ContractError::CustomError(
                "Stake response: reinit_unstake_id == 0 and status is Success, but message have tokens".into()
            ))
            .root_cause()
            .to_string()
        ),
    }
}
