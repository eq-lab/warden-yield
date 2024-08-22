use crate::msg::ExecuteMsg;
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::{to_json_binary, Addr, Uint128, WasmMsg};
use cw_multi_test::{BasicApp, Executor};

use crate::tests::utils::call::call_add_token;
use crate::tests::utils::init::{
    get_lp_contract_address_from_response, get_tokens_info, instantiate_cw20,
    instantiate_yield_ward_contract_without_tokens,
};
use crate::tests::utils::query::get_token_config;
use crate::tests::utils::types::{TestInfo, TokenTestInfo};
use lp_token::msg::QueryMsg as lp_token_query_msg;

fn assert_token_config(
    app: &BasicApp,
    test_info: &TestInfo,
    lpt: &TokenTestInfo,
    actual_lpt_address: &Addr,
) {
    let token_config = get_token_config(app, test_info, &lpt.deposit_token_denom);

    assert_eq!(token_config.is_stake_enabled, lpt.is_stake_enabled);
    assert_eq!(token_config.is_unstake_enabled, lpt.is_unstake_enabled);
    assert_eq!(token_config.symbol, lpt.symbol);
    assert_eq!(token_config.chain, lpt.chain);
    assert_eq!(token_config.evm_yield_contract, lpt.evm_yield_contract);
    assert_eq!(token_config.evm_address, lpt.evm_address);
    assert_eq!(token_config.lp_token_denom, lpt.lp_token_denom);
    assert_eq!(token_config.lp_token_address, actual_lpt_address);
}

#[test]
fn test_add_one_lpt() {
    let (mut app, test_info) = instantiate_yield_ward_contract_without_tokens();

    let tokens = get_tokens_info();
    let lpt0 = tokens.get(0).unwrap();
    let cw20_deposit_token = instantiate_cw20(
        &mut app,
        &test_info,
        test_info.lp_token_code_id,
        &"TestTok".to_owned(),
        &"TestT".to_owned(),
    );
    let resp = call_add_token(&mut app, &test_info, &lpt0, &cw20_deposit_token);

    let actual_lpt_address = get_lp_contract_address_from_response(&resp);
    println!("LP token address from event: {}", actual_lpt_address);

    // check states
    assert_token_config(&app, &test_info, &lpt0, &actual_lpt_address);

    // app.wrap().query_wasm_smart();
    let lp_token_data = app.contract_data(&actual_lpt_address).unwrap();
    assert_eq!(lp_token_data.code_id, test_info.lp_token_code_id);
}

#[test]
fn test_add_two_lpt() {
    let (mut app, test_info) = instantiate_yield_ward_contract_without_tokens();

    let tokens = get_tokens_info();
    let lpt0 = tokens.get(0).unwrap();
    let lpt1 = tokens.get(1).unwrap();

    let cw20_deposit_token_0 = instantiate_cw20(
        &mut app,
        &test_info,
        test_info.lp_token_code_id,
        &"TestTokZero".to_owned(),
        &"TestTZ".to_owned(),
    );
    let cw20_deposit_token_1 = instantiate_cw20(
        &mut app,
        &test_info,
        test_info.lp_token_code_id,
        &"TestTokOne".to_owned(),
        &"TestTO".to_owned(),
    );
    let resp0 = call_add_token(&mut app, &test_info, &lpt0, &cw20_deposit_token_0);
    let resp1 = call_add_token(&mut app, &test_info, &lpt1, &cw20_deposit_token_1);

    let actual_lpt_0_address = get_lp_contract_address_from_response(&resp0);
    let actual_lpt_1_address = get_lp_contract_address_from_response(&resp1);
    println!("LP token 0 address from event: {}", actual_lpt_0_address);
    println!("LP token 1 address from event: {}", actual_lpt_1_address);

    // check states
    assert_token_config(&app, &test_info, &lpt0, &actual_lpt_0_address);
    assert_token_config(&app, &test_info, &lpt1, &actual_lpt_1_address);

    // app.wrap().query_wasm_smart();
    let lp_token_data_0 = app.contract_data(&actual_lpt_0_address).unwrap();
    let lp_token_data_1 = app.contract_data(&actual_lpt_1_address).unwrap();
    assert_eq!(lp_token_data_0.code_id, test_info.lp_token_code_id);
    assert_eq!(lp_token_data_1.code_id, test_info.lp_token_code_id);
}

#[test]
fn test_mint_lpt() {
    let (mut app, test_info) = instantiate_yield_ward_contract_without_tokens();

    let tokens = get_tokens_info();
    let lpt0 = tokens.get(0).unwrap();
    let cw20_deposit_token = instantiate_cw20(
        &mut app,
        &test_info,
        test_info.lp_token_code_id,
        &"TestTok".to_owned(),
        &"TestT".to_owned(),
    );
    let resp = call_add_token(&mut app, &test_info, &lpt0, &cw20_deposit_token);
    let mint_amount = Uint128::new(12345);

    let actual_lpt_address = get_lp_contract_address_from_response(&resp);
    println!("LP token address from event: {}", actual_lpt_address);

    let msg = Wasm(WasmMsg::Execute {
        contract_addr: test_info.yield_ward_address.to_string(),
        msg: to_json_binary(&ExecuteMsg::MintLpToken {
            recipient: test_info.user.clone(),
            lp_token_address: actual_lpt_address.clone(),
            amount: mint_amount,
        })
        .unwrap(),
        funds: vec![],
    });
    app.execute(test_info.admin.clone(), msg).unwrap();

    let balance_after: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            actual_lpt_address.to_string(),
            &lp_token_query_msg::Balance {
                address: test_info.user.to_string(),
            },
        )
        .unwrap();

    assert_eq!(balance_after.balance, mint_amount);
}

#[test]
fn test_disallow_mint() {
    let (mut app, test_info) = instantiate_yield_ward_contract_without_tokens();

    let tokens = get_tokens_info();
    let lpt0 = tokens.get(0).unwrap();
    let cw20_deposit_token = instantiate_cw20(
        &mut app,
        &test_info,
        test_info.lp_token_code_id,
        &"TestTok".to_owned(),
        &"TestT".to_owned(),
    );
    let resp = call_add_token(&mut app, &test_info, &lpt0, &cw20_deposit_token);
    let mint_amount = Uint128::new(12345);

    let actual_lpt_address = get_lp_contract_address_from_response(&resp);
    println!("LP token address from event: {}", actual_lpt_address);

    let disallow_mint_msg = Wasm(WasmMsg::Execute {
        contract_addr: test_info.yield_ward_address.to_string(),
        msg: to_json_binary(&ExecuteMsg::DisallowMint).unwrap(),
        funds: vec![],
    });
    app.execute(test_info.admin.clone(), disallow_mint_msg)
        .unwrap();

    let mint_msg = Wasm(WasmMsg::Execute {
        contract_addr: test_info.yield_ward_address.to_string(),
        msg: to_json_binary(&ExecuteMsg::MintLpToken {
            recipient: test_info.user.clone(),
            lp_token_address: actual_lpt_address.clone(),
            amount: mint_amount,
        })
        .unwrap(),
        funds: vec![],
    });
    let resp = app.execute(test_info.admin.clone(), mint_msg);

    assert!(resp.is_err());
}
