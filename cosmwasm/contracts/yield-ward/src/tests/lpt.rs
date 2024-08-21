use crate::contract::{
    execute as yield_ward_execute, instantiate as yield_ward_instantiate, query as yield_ward_query,
};
use crate::msg::{ExecuteMsg, GetTokensConfigsResponse, InstantiateMsg, QueryMsg};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::{to_json_binary, Addr, Uint128, WasmMsg};
use cw_multi_test::{App, AppResponse, BasicApp, ContractWrapper, Executor};
use lp_token::contract::{
    execute as lp_token_execute, instantiate as lp_token_instantiate, query as lp_token_query,
};
use lp_token::msg::QueryMsg as lp_token_query_msg;

struct TestInfo {
    pub lp_token_code_id: u64,
    pub yield_ward_code_id: u64,
    pub yield_ward_address: Addr,
    pub owner: Addr,
    pub user: Addr,
    pub axelar: Addr,
}

struct LptInfo {
    pub deposit_token_denom: String,
    pub lp_token_denom: String,
    pub is_stake_enabled: bool,
    pub is_unstake_enabled: bool,
    pub symbol: String,
    pub name: String,
    pub chain: String,
    pub evm_yield_contract: String,
    pub evm_address: String,
}

fn get_lpt_0_info() -> LptInfo {
    LptInfo {
        deposit_token_denom: "deposit_token_denom_0".to_string(),
        lp_token_denom: "lp_token_denom_0".to_string(),
        is_stake_enabled: true,
        is_unstake_enabled: true,
        symbol: "LPT-zero".to_string(),
        name: "LP token 0".to_string(),
        chain: "Ethereum".to_string(),
        evm_yield_contract: "0x0000000000000000000000000000000000000077".to_string(),
        evm_address: "0x0000000000000000000000000000000000000007".to_string(),
    }
}

fn get_lpt_1_info() -> LptInfo {
    LptInfo {
        deposit_token_denom: "deposit_token_denom_1".to_string(),
        lp_token_denom: "lp_token_denom_1".to_string(),
        is_stake_enabled: true,
        is_unstake_enabled: true,
        symbol: "LPT-one".to_string(),
        name: "LP token 1".to_string(),
        chain: "Ethereum".to_string(),
        evm_yield_contract: "0x0000000000000000000000000000000000010077".to_string(),
        evm_address: "0x0000000000000000000000000000000000010007".to_string(),
    }
}

fn store_lp_token_code(app: &mut App) -> u64 {
    let lp_token_code =
        ContractWrapper::new(lp_token_execute, lp_token_instantiate, lp_token_query);
    app.store_code(Box::new(lp_token_code))
}

fn store_yield_ward_code(app: &mut App) -> u64 {
    let yield_ward_code =
        ContractWrapper::new(yield_ward_execute, yield_ward_instantiate, yield_ward_query);
    app.store_code(Box::new(yield_ward_code))
}

fn instantiate_yield_ward_contract(app: &mut App) -> TestInfo {
    let lp_token_code_id = store_lp_token_code(app);
    let yield_ward_code_id = store_yield_ward_code(app);

    let owner = app.api().addr_make("owner");
    let user = app.api().addr_make("user");
    let axelar = app.api().addr_make("axelar");

    let yield_ward_address = app
        .instantiate_contract(
            yield_ward_code_id,
            owner.clone(),
            &InstantiateMsg {
                tokens: vec![],
                axelar: axelar.clone(),
                lp_token_code_id,
            },
            &[],
            "YieldWard",
            Some(owner.to_string()),
        )
        .unwrap();

    TestInfo {
        lp_token_code_id,
        yield_ward_code_id,
        yield_ward_address,
        owner,
        user,
        axelar,
    }
}

fn call_add_token(app: &mut BasicApp, test_info: &TestInfo, lpt: &LptInfo) -> AppResponse {
    app.execute(
        test_info.owner.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: test_info.yield_ward_address.to_string(),
            msg: to_json_binary(&ExecuteMsg::AddToken {
                token_denom: lpt.deposit_token_denom.clone(),
                is_stake_enabled: lpt.is_stake_enabled,
                is_unstake_enabled: lpt.is_unstake_enabled,
                symbol: lpt.symbol.clone(),
                name: lpt.name.clone(),
                chain: lpt.chain.clone(),
                evm_yield_contract: lpt.evm_yield_contract.clone(),
                evm_address: lpt.evm_address.clone(),
                lp_token_denom: lpt.lp_token_denom.clone(),
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap()
}

fn get_lp_contract_address_from_response(resp: &AppResponse) -> Addr {
    let instantiate_event = resp.events.iter().find(|x| x.ty == "instantiate").unwrap();
    Addr::unchecked(
        instantiate_event
            .attributes
            .iter()
            .find(|x| x.key == "_contract_address")
            .unwrap()
            .value
            .to_owned(),
    )
}

fn assert_token_config(
    app: &BasicApp,
    test_info: &TestInfo,
    lpt: &LptInfo,
    actual_lpt_address: &Addr,
) {
    let tokens_configs: GetTokensConfigsResponse = app
        .wrap()
        .query_wasm_smart(
            test_info.yield_ward_address.to_string(),
            &QueryMsg::TokensConfigs,
        )
        .unwrap();

    let (_, token_config) = tokens_configs
        .tokens
        .iter()
        .find(|(x, _)| x.to_string() == lpt.deposit_token_denom)
        .unwrap()
        .clone();

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
    let mut app = App::default();
    let test_info = instantiate_yield_ward_contract(&mut app);

    let lpt0 = get_lpt_0_info();
    let resp = call_add_token(&mut app, &test_info, &lpt0);
    // println!("Events: {:?}", resp.events);

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
    let mut app = App::default();
    let test_info = instantiate_yield_ward_contract(&mut app);

    let lpt0 = get_lpt_0_info();
    let lpt1 = get_lpt_1_info();

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
    let mut app = App::default();
    let test_info = instantiate_yield_ward_contract(&mut app);

    let lpt0 = get_lpt_0_info();
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
    app.execute(test_info.owner.clone(), msg).unwrap();

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
    let mut app = App::default();
    let test_info = instantiate_yield_ward_contract(&mut app);

    let lpt0 = get_lpt_0_info();
    let resp = call_add_token(&mut app, &test_info, &lpt0);
    let mint_amount = Uint128::new(12345);

    let actual_lpt_address = get_lp_contract_address_from_response(&resp);
    println!("LP token address from event: {}", actual_lpt_address);

    let disallow_mint_msg = Wasm(WasmMsg::Execute {
        contract_addr: test_info.yield_ward_address.to_string(),
        msg: to_json_binary(&ExecuteMsg::DisallowMint).unwrap(),
        funds: vec![],
    });
    app.execute(test_info.owner.clone(), disallow_mint_msg)
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
    let resp = app.execute(test_info.owner.clone(), mint_msg);

    assert!(resp.is_err());
}
