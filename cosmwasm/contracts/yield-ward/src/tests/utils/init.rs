use crate::msg::InstantiateMsg;
use crate::state::AxelarConfigState;
use crate::tests::utils::call::call_add_token;
use crate::tests::utils::query::get_bank_token_balance;
use crate::tests::utils::types::{TestInfo, TestingApp, TokenTestInfo};
use cosmwasm_std::{Addr, Coin, Uint128};
use cw20::MinterResponse;
use cw_multi_test::{AppBuilder, AppResponse, ContractWrapper, Executor};
use lp_token::contract::{
    execute as lp_token_execute, instantiate as lp_token_instantiate, query as lp_token_query,
    InstantiateMsg as Cw20InstantiateMsg,
};

use crate::tests::mocks::ibc_module_mock::IbcModuleMock;

fn store_lp_token_code(app: &mut TestingApp) -> u64 {
    let lp_token_code =
        ContractWrapper::new(lp_token_execute, lp_token_instantiate, lp_token_query);
    app.store_code(Box::new(lp_token_code))
}

fn store_yield_ward_code(app: &mut TestingApp) -> u64 {
    let yield_ward_code = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    app.store_code(Box::new(yield_ward_code))
}

pub fn _instantiate_cw20(
    app: &mut TestingApp,
    ctx: &TestInfo,
    cw20_code_id: u64,
    name: &String,
    symbol: &String,
) -> Addr {
    let cw20_address = app
        .instantiate_contract(
            cw20_code_id,
            ctx.admin.clone(),
            &Cw20InstantiateMsg {
                name: name.clone(),
                symbol: symbol.clone(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: ctx.admin.to_string(),
                    cap: None,
                }),
                marketing: None,
            },
            &[],
            symbol.clone(),
            Some(ctx.admin.to_string()),
        )
        .unwrap();

    cw20_address
}

pub fn instantiate_yield_ward_contract_without_tokens() -> (TestingApp, TestInfo) {
    let mut app = AppBuilder::new()
        .with_ibc(IbcModuleMock::new())
        .build(cw_multi_test::no_init);

    let lp_token_code_id = store_lp_token_code(&mut app);
    let yield_ward_code_id = store_yield_ward_code(&mut app);

    let admin = app.api().addr_make("admin");
    let user = app.api().addr_make("user");
    let unstake_user = app.api().addr_make("unstake_user");
    let axelar = app.api().addr_make("axelar");

    let yield_ward_address = app
        .instantiate_contract(
            yield_ward_code_id,
            admin.clone(),
            &InstantiateMsg {
                axelar: axelar.clone(),
                lp_token_code_id,
                axelar_config: get_axelar_config(),
            },
            &[],
            "YieldWard",
            Some(admin.to_string()),
        )
        .unwrap();

    let test_info = TestInfo {
        lp_token_code_id,
        yield_ward_address,
        admin,
        user,
        unstake_user,
        axelar,
        tokens: vec![],
    };

    (app, test_info)
}

pub fn instantiate_yield_ward_contract_with_tokens() -> (TestingApp, TestInfo) {
    let admin_str = "admin";
    let user_str = "user";
    let unstake_user_str = "unstake_user";
    let axelar_str = "axelar";
    let deposit_token_mint_amount = Uint128::new(5000000000_u128);

    let tokens = get_tokens_info();
    let mut app = AppBuilder::new()
        .with_ibc(IbcModuleMock::new())
        .build(|router, api, storage| {
            api.addr_make(&admin_str.to_string());
            let user = api.addr_make(&user_str.to_string());
            let unstake_user = api.addr_make(&unstake_user_str.to_string());
            api.addr_make(&axelar_str.to_string());

            let coins: Vec<Coin> = tokens
                .iter()
                .map(|x| Coin {
                    denom: x.deposit_token_denom.to_string(),
                    amount: deposit_token_mint_amount,
                })
                .collect();

            router
                .bank
                .init_balance(storage, &user, coins.clone())
                .unwrap();

            router
                .bank
                .init_balance(storage, &unstake_user, coins)
                .unwrap();
        });

    let admin = app.api().addr_make(&admin_str.to_string());
    let user = app.api().addr_make(&user_str.to_string());
    let unstake_user = app.api().addr_make(&unstake_user_str.to_string());
    let axelar = app.api().addr_make(&axelar_str.to_string());

    for t in &tokens {
        let user_balance = get_bank_token_balance(&app, &t.deposit_token_denom, &user);
        let unstake_user_balance = get_bank_token_balance(&app, &t.deposit_token_denom, &user);

        assert_eq!(user_balance, deposit_token_mint_amount);
        assert_eq!(unstake_user_balance, deposit_token_mint_amount);
    }

    let lp_token_code_id = store_lp_token_code(&mut app);
    let yield_ward_code_id = store_yield_ward_code(&mut app);

    let yield_ward_address = app
        .instantiate_contract(
            yield_ward_code_id,
            admin.clone(),
            &InstantiateMsg {
                axelar: axelar.clone(),
                lp_token_code_id,
                axelar_config: get_axelar_config(),
            },
            &[],
            "YieldWard",
            Some(admin.to_string()),
        )
        .unwrap();

    let test_info = TestInfo {
        lp_token_code_id,
        yield_ward_address: yield_ward_address.clone(),
        admin: admin.clone(),
        user: user.clone(),
        unstake_user: unstake_user.clone(),
        axelar: axelar.clone(),
        tokens,
    };

    for token in &test_info.tokens {
        call_add_token(&mut app, &test_info, &token);
    }

    return (app, test_info);
}

pub fn get_lp_contract_address_from_response(resp: &AppResponse) -> Addr {
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

pub fn get_tokens_info() -> Vec<TokenTestInfo> {
    vec![
        TokenTestInfo {
            deposit_token_denom: "deposit_token_denom_0".to_string(),
            deposit_token_symbol: "DTZERO".to_string(),
            deposit_token_decimals: 18,
            is_stake_enabled: true,
            is_unstake_enabled: true,
            symbol: "LPT-zero".to_string(),
            name: "LP token 0".to_string(),
            chain: super::constants::EVM_DESTINATION_CHAIN_TAG.to_string(),
            evm_yield_contract: super::constants::YIELD_WARD_EVM_ADDRESS.to_string(),
            evm_address: "0x0000000000000000000000000000000000000007".to_string(),
        },
        TokenTestInfo {
            deposit_token_denom: "deposit_token_denom_1".to_string(),
            deposit_token_symbol: "DTONE".to_string(),
            deposit_token_decimals: 6,
            is_stake_enabled: true,
            is_unstake_enabled: true,
            symbol: "LPT-one".to_string(),
            name: "LP token 1".to_string(),
            chain: super::constants::EVM_DESTINATION_CHAIN_TAG.to_string(),
            evm_yield_contract: "0x0000000000000000000000000000000000010077".to_string(),
            evm_address: "0x0000000000000000000000000000000000010007".to_string(),
        },
    ]
}

fn get_axelar_config() -> AxelarConfigState {
    AxelarConfigState {
        axelar_channel_id: super::constants::AXELAR_CHANNEL_ID.into(),
        axelar_gateway_cosmos_address: super::constants::AXELAR_GATEWAY_COSMOS_ADDRESS.into(),
        axelar_fee_recipient_address: super::constants::AXELAR_FEE_RECIPIENT_ADDRESS.into(),
        ibc_timeout_seconds: super::constants::IBC_TIMEOUT_SECONDS,
    }
}
