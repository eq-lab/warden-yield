use crate::helpers::assert_msg_sender_is_admin;
use crate::state::{
    QueueParams, StakeStatsItem, CONTRACT_CONFIG, STAKE_PARAMS, STAKE_STATS, TOKEN_CONFIG,
    TOKEN_DENOM_BY_SOURCE, UNSTAKE_PARAMS,
};
use crate::types::TokenConfig;
use crate::ContractError;
use cosmwasm_std::QueryRequest::Wasm;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, Addr, Binary, CodeInfoResponse, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdError, StdResult, WasmMsg, WasmQuery,
};
use lp_token::msg::QueryMsg::TokenInfo;
use lp_token::msg::{InstantiateMarketingInfo, InstantiateMsg as LpInstantiateMsg};

pub fn try_add_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_denom: String,
    cw20_address: Addr,
    is_stake_enabled: bool,
    is_unstake_enabled: bool,
    chain: String,
    lpt_symbol: String,
    lpt_name: String,
    evm_yield_contract: String,
    evm_address: String,
    lpt_denom: String,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    if TOKEN_CONFIG.has(deps.storage, &token_denom) {
        return Err(ContractError::TokenAlreadyExist(token_denom));
    }

    let contract_config = CONTRACT_CONFIG.load(deps.storage)?;

    STAKE_STATS.save(deps.storage, &token_denom, &StakeStatsItem::default())?;
    STAKE_PARAMS.save(
        deps.storage,
        &token_denom,
        &QueueParams {
            pending_count: 0,
            next_id: 1,
        },
    )?;
    UNSTAKE_PARAMS.save(
        deps.storage,
        &token_denom,
        &QueueParams {
            pending_count: 0,
            next_id: 1,
        },
    )?;

    TOKEN_DENOM_BY_SOURCE.save(deps.storage, (&chain, &evm_yield_contract), &token_denom)?;

    let deposit_token_info: cw20::TokenInfoResponse =
        deps.querier.query(&Wasm(WasmQuery::Smart {
            contract_addr: cw20_address.to_string(),
            msg: to_json_binary(&TokenInfo {})?,
        }))?;

    let msg = to_json_binary(&LpInstantiateMsg {
        name: lpt_name,
        symbol: lpt_symbol.clone(),
        decimals: deposit_token_info.decimals,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: env.contract.address.to_string(),
            cap: None,
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("YieldWard".to_string()),
            description: Some("LP token".to_string()),
            marketing: None,
            logo: None,
        }),
    })?;

    let tokens = TOKEN_CONFIG
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    let salt = Binary::new(Vec::from(tokens.to_be_bytes()));
    let inst2_msg = WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: contract_config.lp_token_code_id,
        label: "LP token".to_string(),
        msg,
        funds: vec![],
        salt: salt.clone(),
    };

    let lpt_address =
        calculate_token_address(deps.as_ref(), env, contract_config.lp_token_code_id, salt)?;

    TOKEN_CONFIG.save(
        deps.storage,
        &token_denom,
        &TokenConfig {
            cw20_address,
            deposit_token_symbol: deposit_token_info.symbol,
            is_stake_enabled,
            is_unstake_enabled,
            chain,
            evm_yield_contract,
            evm_address,
            lpt_symbol,
            lpt_denom,
            lpt_address,
        },
    )?;

    Ok(Response::new().add_message(inst2_msg))
}

pub fn calculate_token_address(
    deps: Deps,
    env: Env,
    code_id: u64,
    salt: Binary,
) -> StdResult<Addr> {
    let canonical_creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;

    let code_info: CodeInfoResponse = deps.querier.query(&Wasm(WasmQuery::CodeInfo { code_id }))?;
    let canonical_addr =
        instantiate2_address(code_info.checksum.as_slice(), &canonical_creator, &salt)
            .map_err(|_| StdError::generic_err("Could not calculate addr"))?;

    deps.api.addr_humanize(&canonical_addr)
}