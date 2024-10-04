use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::{to_json_binary, WasmMsg};
use cw_multi_test::error::anyhow;
use cw_multi_test::Executor;
use lp_token::ContractError;

use crate::{
    msg::{ExecuteMsg, GetContractConfigResponse, QueryMsg},
    state::ContractConfigState,
};

use super::utils::init::instantiate_yield_ward_contract_with_tokens;

#[test]
fn test_update_contract_config() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let old_contract_config_response: GetContractConfigResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::ContractConfig {},
        )
        .unwrap();

    let old_contract_config = old_contract_config_response.config;

    let contract_config = ContractConfigState {
        owner: old_contract_config.axelar,
        axelar: old_contract_config.owner,
        lp_token_code_id: old_contract_config.lp_token_code_id + 1,
        is_mint_allowed: !old_contract_config.is_mint_allowed,
    };

    app.execute(
        ctx.admin,
        Wasm(WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: to_json_binary(&ExecuteMsg::UpdateContractConfig {
                contract_config: contract_config.clone(),
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap();

    let new_contract_config_response: GetContractConfigResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::ContractConfig {},
        )
        .unwrap();

    assert_eq!(contract_config, new_contract_config_response.config);
}

#[test]
fn test_update_contract_config_only_admin() {
    let (mut app, ctx) = instantiate_yield_ward_contract_with_tokens();

    let old_contract_config_response: GetContractConfigResponse = app
        .wrap()
        .query_wasm_smart(
            ctx.yield_ward_address.to_string(),
            &QueryMsg::ContractConfig {},
        )
        .unwrap();

    let old_contract_config = old_contract_config_response.config;

    let contract_config = ContractConfigState {
        owner: old_contract_config.axelar,
        axelar: old_contract_config.owner,
        lp_token_code_id: old_contract_config.lp_token_code_id + 1,
        is_mint_allowed: !old_contract_config.is_mint_allowed,
    };

    assert_ne!(ctx.admin, ctx.user);

    match app.execute(
        ctx.user,
        Wasm(WasmMsg::Execute {
            contract_addr: ctx.yield_ward_address.to_string(),
            msg: to_json_binary(&ExecuteMsg::UpdateContractConfig {
                contract_config: contract_config.clone(),
            })
            .unwrap(),
            funds: vec![],
        }),
    ) {
        Ok(_) => panic!("Non-admin successfully updated contract config"),
        Err(err) => {
            assert_eq!(
                err.root_cause().to_string(),
                anyhow!(ContractError::Unauthorized {})
                    .root_cause()
                    .to_string()
            );
        }
    };
}
