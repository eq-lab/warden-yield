use crate::state::{QueueParams, StakeStatsItem, UnstakeItem};
use crate::tests::utils::call::{call_reinit, call_reinit_response, call_stake_and_unstake};
use crate::tests::utils::init::instantiate_yield_ward_contract_with_tokens;
use crate::tests::utils::query::{get_stake_stats, get_unstake_item, get_unstake_params};
use crate::types::{ReinitResponseData, UnstakeActionStage};
use crate::ContractError;
use cosmwasm_std::{coins, Binary, Uint256};
use cw_multi_test::error::anyhow;
use cw_multi_test::Executor;

use super::utils::calldata::create_reinit_response_payload;

#[test]
fn test_reinit() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.get(0).unwrap();
    let unstake_user = &ctx.unstake_user;

    let unstake_details = call_stake_and_unstake(&mut app, &ctx, unstake_user, &token_info);

    call_reinit(&mut app, &ctx, unstake_user, token_info);

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

#[test]
fn test_reinit_wrong_funds() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.get(0).unwrap();

    let coin = cosmwasm_std::coin(
        crate::tests::utils::constants::AXELAR_FEE,
        token_info.deposit_token_denom.to_string(),
    );
    let funds = vec![coin.clone(), coin];

    let reinit_result = app.execute(
        ctx.unstake_user,
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::Reinit {
                token_denom: token_info.deposit_token_denom.to_string(),
            })
            .unwrap(),
            funds,
        }),
    );

    match reinit_result {
        Ok(_) => panic!("Reinit passed with wrong funds length"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            "Custom Error: Wrong number of tokens attached to reinit call"
        ),
    }
}

#[test]
fn test_wrong_reinit_response() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();
    let token_info = ctx.tokens.get(0).unwrap();

    let coin = cosmwasm_std::coin(
        crate::tests::utils::constants::AXELAR_FEE,
        token_info.deposit_token_denom.to_string(),
    );
    let funds = vec![coin.clone(), coin.clone()];

    let response_payload = create_reinit_response_payload(ReinitResponseData {
        reinit_unstake_id: 1,
    });

    let response_wrong_funds = app.execute(
        ctx.axelar.clone(),
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::HandleResponse {
                source_chain: token_info.chain.to_string(),
                source_address: token_info.evm_yield_contract.to_string(),
                payload: response_payload.clone(),
            })
            .unwrap(),
            funds: funds.clone(),
        }),
    );

    match response_wrong_funds {
        Ok(_) => panic!("Reinit response passed with wrong funds length"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            anyhow!(ContractError::CustomError(
                "Reinit message must have one type of coins as funds".into()
            ))
            .root_cause()
            .to_string()
        ),
    }

    let wrong_denom_value = ctx.tokens.get(1).unwrap().deposit_token_denom.clone();

    let wrong_denom = app.execute(
        ctx.axelar.clone(),
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::HandleResponse {
                source_chain: token_info.chain.to_string(),
                source_address: token_info.evm_yield_contract.to_string(),
                payload: response_payload.clone(),
            })
            .unwrap(),
            funds: coins(100, wrong_denom_value.clone()),
        }),
    );

    match wrong_denom {
        Ok(_) => panic!("Reinit response passed with wrong denom"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            anyhow!(ContractError::InvalidToken {
                actual: wrong_denom_value,
                expected: coin.denom
            })
            .root_cause()
            .to_string()
        ),
    }

    let invalid_payload = app.execute(
        ctx.axelar,
        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: cosmwasm_std::to_json_binary(&crate::msg::ExecuteMsg::HandleResponse {
                source_chain: token_info.chain.to_string(),
                source_address: token_info.evm_yield_contract.to_string(),
                payload: Binary::from([0]),
            })
            .unwrap(),
            funds: funds,
        }),
    );

    match invalid_payload {
        Ok(_) => panic!("Reinit response passed with invalid payload"),
        Err(err) => assert_eq!(
            err.root_cause().to_string(),
            anyhow!(ContractError::InvalidMessagePayload {})
                .root_cause()
                .to_string()
        ),
    }
}
