use crate::contract::{
    execute as yield_ward_execute, instantiate as yield_ward_instantiate, query as yield_ward_query,
};
use crate::msg::{ExecuteMsg, GetTokensConfigsResponse, InstantiateMsg, QueryMsg};
use cosmwasm_std::testing::message_info;

use crate::tests::utils::get_token_config;
use crate::types::TokenConfig;
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmQuery::Smart;
use cosmwasm_std::{to_json_binary, Addr, QueryRequest, WasmMsg};
use cw_multi_test::{App, ContractWrapper, Executor};
use lp_token::contract::{
    execute as lp_token_execute, instantiate as lp_token_instantiate, query as lp_token_query,
};

struct TestInfo {
    pub lp_token_code_id: u64,
    pub yield_ward_code_id: u64,
    pub yield_ward_address: Addr,
    pub owner: Addr,
    pub axelar: Addr,
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

    let owner = Addr::unchecked("owner");
    let axelar = Addr::unchecked("axelar");

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
        axelar,
    }
}
#[test]
fn test_init_stake_one_coin() {
    let mut app = App::default();
    let test_info = instantiate_yield_ward_contract(&mut app);

    let deposit_token_denom = "deposit_token_denom".to_string();
    let lp_token_denom = "lp_token_denom".to_string();
    let is_stake_enabled = true;
    let is_unstake_enabled = true;
    let symbol = "LPT".to_string();
    let name = "LP token 1".to_string();
    let chain = "Ethereum".to_string();
    let evm_yield_contract = "0x0000000000000000000000000000000000000077".to_string();
    let evm_address = "0x0000000000000000000000000000000000000007".to_string();

    // let lp_token_address = calculate_token_address(ctx.deps.as_ref(), ctx.env.clone()).unwrap();

    let resp = app
        .execute(
            test_info.owner.clone(),
            Wasm(WasmMsg::Execute {
                contract_addr: test_info.yield_ward_address.to_string(),
                msg: to_json_binary(&ExecuteMsg::AddToken {
                    token_denom: deposit_token_denom.clone(),
                    is_stake_enabled,
                    is_unstake_enabled,
                    symbol: symbol.clone(),
                    name: name.clone(),
                    chain: chain.clone(),
                    evm_yield_contract: evm_yield_contract.clone(),
                    evm_address: evm_address.clone(),
                    lp_token_denom: lp_token_denom.clone(),
                })
                .unwrap(),
                funds: vec![],
            }),
        )
        .unwrap();

    println!("Events: {:?}", resp.events);
    let instantiate_event = resp.events.iter().find(|x| x.ty == "instantiate").unwrap();
    let lp_token_address = instantiate_event
        .attributes
        .iter()
        .find(|x| x.key == "_contract_address")
        .unwrap()
        .value
        .to_owned();

    println!("LP token address from event: {}", lp_token_address);

    // check states
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
        .find(|(x, _)| x.to_string() == deposit_token_denom)
        .unwrap()
        .clone();

    assert_eq!(token_config.is_stake_enabled, is_stake_enabled);
    assert_eq!(token_config.is_unstake_enabled, is_unstake_enabled);
    assert_eq!(token_config.symbol, symbol);
    assert_eq!(token_config.chain, chain);
    assert_eq!(token_config.evm_yield_contract, evm_yield_contract);
    assert_eq!(token_config.evm_address, evm_address);
    assert_eq!(token_config.lp_token_denom, lp_token_denom);
    // todo: fix different addresses
    assert_eq!(token_config.lp_token_address.to_string(), lp_token_address);

    // app.wrap().query_wasm_smart();
    let lp_token_data = app.contract_data(&token_config.lp_token_address).unwrap();
    assert_eq!(lp_token_data.code_id, test_info.lp_token_code_id);
    // todo: check events ?
}
