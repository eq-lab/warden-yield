use crate::helpers::assert_msg_sender_is_admin;
use crate::state::{
    QueueParams, StakeStatsItem, CONTRACT_CONFIG, STAKE_PARAMS, STAKE_STATS, TOKEN_CONFIG,
    TOKEN_DENOM_BY_LPT_ADDRESS, TOKEN_DENOM_BY_SOURCE, UNSTAKE_PARAMS,
};
use crate::types::{TokenConfig, TokenDenom};
use crate::ContractError;
use cosmwasm_std::QueryRequest::Wasm;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, Addr, Binary, CodeInfoResponse, Deps, DepsMut, Env,
    Event, MessageInfo, Order, Response, StdError, StdResult, WasmMsg, WasmQuery,
};
use lp_token::msg::{InstantiateMarketingInfo, InstantiateMsg as LpInstantiateMsg};

pub fn try_add_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_denom: TokenDenom,
    token_symbol: String,
    token_decimals: u8,
    is_stake_enabled: bool,
    is_unstake_enabled: bool,
    chain: String,
    lpt_symbol: String,
    lpt_name: String,
    evm_yield_contract: String,
    evm_address: String,
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

    TOKEN_DENOM_BY_SOURCE.save(
        deps.storage,
        (&chain, &evm_yield_contract.to_lowercase()),
        &token_denom,
    )?;

    let msg = to_json_binary(&LpInstantiateMsg {
        name: lpt_name.clone(),
        symbol: lpt_symbol.clone(),
        decimals: token_decimals,
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

    TOKEN_DENOM_BY_LPT_ADDRESS.save(deps.storage, &lpt_address, &token_denom)?;

    TOKEN_CONFIG.save(
        deps.storage,
        &token_denom,
        &TokenConfig {
            deposit_token_symbol: token_symbol,
            is_stake_enabled,
            is_unstake_enabled,
            chain: chain.clone(),
            evm_yield_contract: evm_yield_contract.to_lowercase(),
            evm_address: evm_address.to_lowercase(),
            lpt_symbol: lpt_symbol.clone(),
            lpt_address: lpt_address.clone(),
        },
    )?;

    Ok(Response::new()
        .add_event(
            Event::new("add_token")
                .add_attribute("lpt_symbol", lpt_symbol)
                .add_attribute("lpt_name", lpt_name)
                .add_attribute("lpt_address", lpt_address)
                .add_attribute("decimals", token_decimals.to_string())
                .add_attribute("token_denom", token_denom)
                .add_attribute("chain", chain)
                .add_attribute("yield_contract", evm_yield_contract),
        )
        .add_message(inst2_msg))
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
