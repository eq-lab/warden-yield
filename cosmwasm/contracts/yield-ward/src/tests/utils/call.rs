use crate::msg::{Cw20ActionMsg, ExecuteMsg};
use crate::state::{QueueParams, StakeStatsItem, UnstakeItem};
use crate::tests::utils::calldata::{
    create_reinit_response_payload, create_stake_response_payload, create_unstake_response_payload,
};
use crate::tests::utils::query::{
    get_cw20_balance, get_stake_item, get_stake_params, get_stake_stats, get_token_config,
    get_unstake_item, get_unstake_params,
};
use crate::tests::utils::types::{TestInfo, TokenTestInfo, UnstakeDetails};
use crate::types::{
    ReinitResponseData, StakeResponseData, Status, TokenConfig, TokenDenom, UnstakeActionStage,
    UnstakeResponseData,
};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::{to_json_binary, Addr, Uint128, Uint256, WasmMsg};
use cw20::Cw20ExecuteMsg;
use cw_multi_test::{AppResponse, BasicApp, Executor};

pub fn call_add_token(
    app: &mut BasicApp,
    test_info: &TestInfo,
    lpt: &TokenTestInfo,
    cw20_address: &Addr,
) -> AppResponse {
    app.execute(
        test_info.admin.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: test_info.yield_ward_address.to_string(),
            msg: to_json_binary(&ExecuteMsg::AddToken {
                token_denom: lpt.deposit_token_denom.clone(),
                cw20_address: cw20_address.clone(),
                is_stake_enabled: lpt.is_stake_enabled,
                is_unstake_enabled: lpt.is_unstake_enabled,
                lpt_symbol: lpt.symbol.clone(),
                lpt_name: lpt.name.clone(),
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

pub fn call_update_token_config(
    app: &mut BasicApp,
    test_info: &TestInfo,
    token_denom: &TokenDenom,
    token_config: &TokenConfig,
) -> AppResponse {
    app.execute(
        test_info.admin.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: test_info.yield_ward_address.to_string(),
            msg: to_json_binary(&ExecuteMsg::UpdateTokenConfig {
                token_denom: token_denom.clone(),
                config: token_config.clone(),
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap()
}

pub fn call_mint_cw20(
    app: &mut BasicApp,
    ctx: &TestInfo,
    cw20_address: &Addr,
    recipient: &Addr,
    amount: Uint128,
) {
    let balance_before = get_cw20_balance(app, &cw20_address, recipient);
    app.execute(
        ctx.admin.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: cw20_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: recipient.to_string(),
                amount,
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap();

    let balance_after = get_cw20_balance(app, &cw20_address, recipient);

    assert_eq!(balance_after, balance_before + amount);
}

pub fn call_stake(
    app: &mut BasicApp,
    ctx: &TestInfo,
    from: &Addr,
    token_info: &TokenTestInfo,
    amount: Uint128,
) {
    let token_config = get_token_config(&app, &ctx, &token_info.deposit_token_denom);
    let balance_before = get_cw20_balance(app, &token_config.cw20_address, &from);
    assert!(balance_before >= amount);

    app.execute(
        from.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: token_config.cw20_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: ctx.yield_ward_address.to_string(),
                amount,
                msg: to_json_binary(&Cw20ActionMsg::Stake {
                    deposit_token_denom: token_info.deposit_token_denom.clone(),
                })
                .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap();
}

pub fn call_unstake(
    app: &mut BasicApp,
    ctx: &TestInfo,
    from: &Addr,
    token_info: &TokenTestInfo,
    lpt_amount: Uint128,
) {
    let token_config = get_token_config(&app, &ctx, &token_info.deposit_token_denom);
    let balance_before = get_cw20_balance(app, &token_config.lpt_address, &from);
    assert!(balance_before >= lpt_amount);

    app.execute(
        from.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: token_config.lpt_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: ctx.yield_ward_address.to_string(),
                amount: lpt_amount,
                msg: to_json_binary(&Cw20ActionMsg::Unstake {
                    deposit_token_denom: token_info.deposit_token_denom.clone(),
                })
                .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap();
}

pub fn call_stake_response(
    app: &mut BasicApp,
    ctx: &TestInfo,
    token_info: &TokenTestInfo,
    status: Status,
    stake_id: u64,
    reinit_unstake_id: u64,
    reinit_token_amount: Uint128,
    lp_token_amount: Uint128,
) {
    let token_config = get_token_config(&app, &ctx, &token_info.deposit_token_denom);

    let mut return_amount = reinit_token_amount;
    if status == Status::Fail {
        return_amount = get_stake_item(app, ctx, &token_info.deposit_token_denom, stake_id)
            .unwrap()
            .token_amount;
    }

    if !return_amount.is_zero() {
        call_mint_cw20(
            app,
            ctx,
            &token_config.cw20_address,
            &ctx.axelar,
            return_amount,
        );
    }

    let response_payload = create_stake_response_payload(StakeResponseData {
        status,
        stake_id,
        reinit_unstake_id,
        lp_token_amount,
    });

    app.execute(
        ctx.axelar.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: token_config.cw20_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: ctx.yield_ward_address.to_string(),
                amount: return_amount,
                msg: to_json_binary(&Cw20ActionMsg::HandleResponse {
                    deposit_token_denom: token_info.deposit_token_denom.clone(),
                    source_chain: token_info.chain.to_string(),
                    source_address: token_info.evm_yield_contract.to_string(),
                    payload: response_payload,
                })
                .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap();
}

pub fn call_unstake_response(
    app: &mut BasicApp,
    ctx: &TestInfo,
    token_info: &TokenTestInfo,
    status: Status,
    unstake_id: u64,
    reinit_unstake_id: u64,
    unstake_amount: Uint128,
) {
    let token_config = get_token_config(&app, &ctx, &token_info.deposit_token_denom);

    if !unstake_amount.is_zero() {
        call_mint_cw20(
            app,
            ctx,
            &token_config.cw20_address,
            &ctx.axelar,
            unstake_amount,
        );
    }

    let response_payload = create_unstake_response_payload(UnstakeResponseData {
        status,
        unstake_id,
        reinit_unstake_id,
    });

    app.execute(
        ctx.axelar.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: token_config.cw20_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: ctx.yield_ward_address.to_string(),
                amount: unstake_amount,
                msg: to_json_binary(&Cw20ActionMsg::HandleResponse {
                    deposit_token_denom: token_info.deposit_token_denom.clone(),
                    source_chain: token_info.chain.to_string(),
                    source_address: token_info.evm_yield_contract.to_string(),
                    payload: response_payload,
                })
                .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap();
}

pub fn call_reinit_response(
    app: &mut BasicApp,
    ctx: &TestInfo,
    token_info: &TokenTestInfo,
    reinit_unstake_id: u64,
    unstake_amount: Uint128,
) {
    assert!(!unstake_amount.is_zero());
    let token_config = get_token_config(&app, &ctx, &token_info.deposit_token_denom);

    call_mint_cw20(
        app,
        ctx,
        &token_config.cw20_address,
        &ctx.axelar,
        unstake_amount,
    );

    let response_payload = create_reinit_response_payload(ReinitResponseData { reinit_unstake_id });

    app.execute(
        ctx.axelar.clone(),
        Wasm(WasmMsg::Execute {
            contract_addr: token_config.cw20_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: ctx.yield_ward_address.to_string(),
                amount: unstake_amount,
                msg: to_json_binary(&Cw20ActionMsg::HandleResponse {
                    deposit_token_denom: token_info.deposit_token_denom.clone(),
                    source_chain: token_info.chain.to_string(),
                    source_address: token_info.evm_yield_contract.to_string(),
                    payload: response_payload,
                })
                .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }),
    )
    .unwrap();
}

pub fn call_stake_and_unstake(
    app: &mut BasicApp,
    ctx: &TestInfo,
    user: &Addr,
    token_info: &TokenTestInfo,
) -> UnstakeDetails {
    let stake_id = get_stake_params(app, ctx, &token_info.deposit_token_denom).next_id;

    let stake_amount = Uint128::from(14000_u128);

    // init stake
    call_stake(app, ctx, &user, token_info, stake_amount);

    let reinit_unstake_id = 0;
    let lp_token_amount = Uint128::from(1001_u128);

    // response for stake action
    call_stake_response(
        app,
        ctx,
        token_info,
        Status::Success,
        stake_id,
        reinit_unstake_id,
        Uint128::zero(),
        lp_token_amount,
    );

    let unstake_id = get_unstake_params(app, ctx, &token_info.deposit_token_denom).next_id;
    call_unstake(app, ctx, &user, token_info, lp_token_amount);

    // response for unstake action
    call_unstake_response(
        app,
        ctx,
        token_info,
        Status::Success,
        unstake_id,
        0,
        Uint128::new(0),
    );

    let unstake_params = get_unstake_params(app, ctx, &token_info.deposit_token_denom);
    assert_eq!(
        unstake_params,
        QueueParams {
            pending_count: 1_u64,
            next_id: 2_u64,
        }
    );

    let unstake_item =
        get_unstake_item(app, ctx, &token_info.deposit_token_denom, unstake_id).unwrap();
    assert_eq!(
        unstake_item,
        UnstakeItem {
            user: user.clone(),
            lp_token_amount,
            action_stage: UnstakeActionStage::Registered,
            token_amount: None
        }
    );

    let stake_stats = get_stake_stats(app, ctx, &token_info.deposit_token_denom);
    assert_eq!(
        stake_stats,
        StakeStatsItem {
            pending_stake: Uint256::zero(),
            lp_token_amount: Uint256::zero(),
            pending_unstake_lp_token_amount: Uint256::from(lp_token_amount),
        }
    );

    UnstakeDetails {
        _stake_id: stake_id,
        _stake_amount: stake_amount,
        unstake_id,
        lp_token_amount,
        unstake_token_amount: stake_amount + Uint128::from(100_u128),
    }
}
