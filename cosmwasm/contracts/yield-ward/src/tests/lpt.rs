use crate::msg::ExecuteMsg;
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::{to_json_binary, Addr, Uint128, WasmMsg};
use cw_multi_test::Executor;

use crate::tests::utils::call::{call_add_token, call_update_token_config};
use crate::tests::utils::init::{
    get_lp_contract_address_from_response, get_tokens_info,
    instantiate_yield_ward_contract_with_tokens, instantiate_yield_ward_contract_without_tokens,
};
use crate::tests::utils::query::{
    get_all_tokens_configs, get_token_config, get_token_denom_by_lpt_address,
    get_token_denom_by_source,
};
use crate::tests::utils::types::{TestInfo, TestingApp, TokenTestInfo};
use lp_token::msg::QueryMsg as lp_token_query_msg;

fn assert_token_config(
    app: &TestingApp,
    test_info: &TestInfo,
    lpt: &TokenTestInfo,
    actual_lpt_address: &Addr,
) {
    let token_config = get_token_config(app, test_info, &lpt.deposit_token_denom);
    assert_eq!(token_config.deposit_token_symbol, lpt.deposit_token_symbol);
    assert_eq!(token_config.is_stake_enabled, lpt.is_stake_enabled);
    assert_eq!(token_config.is_unstake_enabled, lpt.is_unstake_enabled);
    assert_eq!(token_config.chain, lpt.chain);
    assert_eq!(token_config.evm_yield_contract, lpt.evm_yield_contract);
    assert_eq!(token_config.evm_address, lpt.evm_address);
    assert_eq!(token_config.lpt_symbol, lpt.symbol);
    assert_eq!(token_config.lpt_address, actual_lpt_address);
}

#[test]
fn test_add_one_lpt() {
    let (mut app, test_info) = instantiate_yield_ward_contract_without_tokens();

    let tokens = get_tokens_info();
    let lpt0 = tokens.get(0).unwrap();
    let resp = call_add_token(&mut app, &test_info, &lpt0);

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
    let resp0 = call_add_token(&mut app, &test_info, &lpt0);
    let resp1 = call_add_token(&mut app, &test_info, &lpt1);

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
    let resp = call_add_token(&mut app, &test_info, &lpt0);
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
    let resp = call_add_token(&mut app, &test_info, &lpt0);
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

#[test]
fn test_update_token_config() {
    let (mut app, test_info) = instantiate_yield_ward_contract_with_tokens();
    let tokens_configs_before = get_all_tokens_configs(&app, &test_info);
    let token_denom_by_source_before = get_token_denom_by_source(&app, &test_info);
    let token_denom_by_lpt_address_before = get_token_denom_by_lpt_address(&app, &test_info);

    let token_denom = test_info
        .tokens
        .first()
        .unwrap()
        .deposit_token_denom
        .clone();

    let token_config_old = tokens_configs_before[&token_denom].clone();
    let mut token_config_new = tokens_configs_before[&token_denom].clone();
    token_config_new.evm_yield_contract = "new_yield_contract_address".to_string();
    token_config_new.deposit_token_symbol = "new_deposit_token_symbol".to_string();
    token_config_new.lpt_address = Addr::unchecked("warden135");

    call_update_token_config(&mut app, &test_info, &token_denom, &token_config_new);
    let tokens_configs_after = get_all_tokens_configs(&app, &test_info);

    let mut tokens_configs_expected = tokens_configs_before.clone();
    tokens_configs_expected.insert(token_denom.clone(), token_config_new.clone());
    assert_eq!(tokens_configs_expected, tokens_configs_after);

    // check token_denom_by_source storage
    let token_denom_by_source_after = get_token_denom_by_source(&app, &test_info);
    let mut token_denom_by_source_expected = token_denom_by_source_before.clone();
    token_denom_by_source_expected
        .remove(&(token_config_old.chain, token_config_old.evm_yield_contract));

    token_denom_by_source_expected.insert(
        (token_config_new.chain, token_config_new.evm_yield_contract),
        token_denom.clone(),
    );
    assert_eq!(token_denom_by_source_expected, token_denom_by_source_after);

    // check token_denom_by_lpt_address storage
    let token_denom_by_lpt_address_after = get_token_denom_by_lpt_address(&app, &test_info);
    let mut token_denom_by_lpt_address_expected = token_denom_by_lpt_address_before.clone();
    token_denom_by_lpt_address_expected.remove(&token_config_old.lpt_address);

    token_denom_by_lpt_address_expected.insert(token_config_new.lpt_address, token_denom.clone());
    assert_eq!(
        token_denom_by_lpt_address_expected,
        token_denom_by_lpt_address_after
    );
}
