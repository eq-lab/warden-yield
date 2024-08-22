use crate::encoding::{
    decode_payload_action_type, decode_reinit_response_payload, decode_stake_response_payload,
    decode_unstake_response_payload, encode_reinit_payload, encode_stake_payload,
    encode_unstake_payload,
};
use crate::helpers::{
    assert_msg_sender_is_admin, assert_msg_sender_is_axelar, find_token_by_message_source,
};
use crate::msg::Cw20ActionMsg;
use crate::state::{
    QueueParams, StakeItem, StakeStatsItem, UnstakeItem, CONTRACT_CONFIG, STAKES, STAKE_PARAMS,
    STAKE_STATS, TOKEN_CONFIG, TOKEN_DENOM_BY_SOURCE, UNSTAKES, UNSTAKE_PARAMS,
};
use crate::types::{
    ActionType, StakeActionStage, StakeResponseData, Status, TokenConfig, TokenDenom,
    UnstakeActionStage, UnstakeResponseData,
};
use crate::ContractError;
use cosmwasm_std::QueryRequest::Wasm;
use cosmwasm_std::{
    from_json, instantiate2_address, to_hex, to_json_binary, Addr, Attribute, Binary,
    CodeInfoResponse, Deps, DepsMut, Env, Event, MessageInfo, Order, Response, StdError, StdResult,
    Uint128, Uint256, WasmMsg, WasmQuery,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use lp_token::msg::{InstantiateMarketingInfo, InstantiateMsg as LpInstantiateMsg};

pub fn try_init_stake(
    deps: DepsMut,
    _env: Env,
    // info: MessageInfo,
    user: Addr,
    token_denom: TokenDenom,
    token_amount: Uint128,
) -> Result<Response, ContractError> {
    // if info.funds.len() != 1 {
    //     return Err(ContractError::CustomError(
    //         "Init stake message must have one type of coins as funds".to_string(),
    //     ));
    // }
    // let coin = info.funds.first().unwrap();
    if token_amount.is_zero() {
        return Err(ContractError::ZeroTokenAmount);
    }
    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &token_denom)?
        .ok_or(ContractError::UnknownToken(token_denom.clone()))?;

    // check is staking enabled
    if !token_config.is_stake_enabled {
        return Err(ContractError::StakeDisabled(token_config.symbol));
    }

    let stake_params = STAKE_PARAMS.load(deps.storage, &token_denom)?;
    let stake_id = stake_params.next_id;

    // push to stakes map
    STAKES.save(
        deps.storage,
        (&token_denom, stake_id),
        &StakeItem {
            user,
            token_amount,
            action_stage: StakeActionStage::WaitingExecution,
        },
    )?;

    // increment stake next_id
    STAKE_PARAMS.save(
        deps.storage,
        &token_denom,
        &QueueParams {
            pending_count: stake_params.pending_count + 1,
            next_id: stake_id + 1,
        },
    )?;

    // update stake stats
    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    stake_stats.pending_stake += Uint256::from(token_amount);
    STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

    let payload = encode_stake_payload(&stake_id);
    // todo: send tokens to Axelar
    let payload_hex_str = to_hex(payload);

    Ok(Response::new().add_event(
        Event::new("stake")
            .add_attribute("token_symbol", token_config.symbol)
            .add_attribute("evm_yield_contract", token_config.evm_yield_contract)
            .add_attribute("dest_chain", token_config.chain)
            .add_attribute("token_amount", token_amount)
            .add_attribute("payload", "0x".to_owned() + &payload_hex_str),
    ))
}

pub fn try_init_unstake(
    deps: DepsMut,
    _env: Env,
    user: Addr,
    token_denom: TokenDenom,
    lpt_amount: Uint128,
) -> Result<Response, ContractError> {
    // if info.funds.len() != 1 {
    //     return Err(ContractError::CustomError(
    //         "Init unstake message must have one type of coins as funds".to_string(),
    //     ));
    // }
    //
    // let coin = info.funds.first().unwrap();

    // let (token_denom, token_config) = find_token_by_lp_token_denom(deps.as_ref(), &token_denom)?;

    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &token_denom)?
        .ok_or(ContractError::UnknownToken(token_denom.clone()))?;

    // update unstake params
    let mut unstake_params = UNSTAKE_PARAMS.load(deps.storage, &token_denom)?;
    let unstake_id = unstake_params.next_id;
    unstake_params.pending_count += 1;
    unstake_params.next_id += 1;
    UNSTAKE_PARAMS.save(deps.storage, &token_denom, &unstake_params)?;

    // push item to unstakes map
    UNSTAKES.save(
        deps.storage,
        (&token_denom, unstake_id),
        &UnstakeItem {
            user,
            lp_token_amount: lpt_amount,
            action_stage: UnstakeActionStage::WaitingRegistration,
        },
    )?;

    // update stake stats
    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    stake_stats.pending_unstake_lp_token_amount += Uint256::from(lpt_amount);
    STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

    let unstake_payload = encode_unstake_payload(&unstake_id, &lpt_amount);
    // todo: send message to Axelar

    let payload_hex_str = to_hex(unstake_payload);

    Ok(Response::new()
        .add_event(
            Event::new("unstake")
                .add_attribute("token_symbol", token_config.symbol)
                .add_attribute("evm_yield_contract", token_config.evm_yield_contract)
                .add_attribute("dest_chain", token_config.chain)
                .add_attribute("lpt_amount", lpt_amount)
                .add_attribute("payload", "0x".to_owned() + &payload_hex_str),
        )
        .add_message(WasmMsg::Execute {
            contract_addr: token_config.lp_token_address.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Burn { amount: lpt_amount }).unwrap(),
            funds: vec![],
        }))
}

pub fn try_reinit(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    token_denom: TokenDenom,
) -> Result<Response, ContractError> {
    let _token_config = TOKEN_CONFIG.load(deps.storage, &token_denom)?;

    let _reinit_payload = encode_reinit_payload();

    // todo: send message to axelar

    Ok(Response::new())
}

pub fn try_handle_response(
    deps: DepsMut,
    env: Env,
    // info: MessageInfo,
    sender: Addr,
    source_chain: String,
    source_address: String,
    payload: Binary,
    token_amount: Uint128,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_axelar(deps.as_ref(), &sender)?;

    let action_type =
        decode_payload_action_type(&payload).ok_or(ContractError::InvalidActionType)?;

    // skip ActionId first byte
    let payload = &payload[1..];
    match action_type {
        ActionType::Stake => try_handle_stake_response(
            deps,
            env,
            source_chain,
            source_address,
            payload,
            token_amount,
        ),
        ActionType::Unstake => try_handle_unstake_response(
            deps,
            env,
            source_chain,
            source_address,
            payload,
            token_amount,
        ),
        ActionType::Reinit => try_handle_reinit_response(
            deps,
            env,
            source_chain,
            source_address,
            payload,
            token_amount,
        ),
    }
}

pub fn try_handle_stake_response(
    deps: DepsMut,
    _env: Env,
    // info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: &[u8],
    token_amount: Uint128,
) -> Result<Response, ContractError> {
    let stake_response =
        decode_stake_response_payload(payload).ok_or(ContractError::InvalidMessagePayload)?;

    let (token_denom, token_config) =
        find_token_by_message_source(deps.as_ref(), &source_chain, &source_address)?;

    ensure_stake_response_is_valid(token_amount, &token_denom, &stake_response)?;

    let mut stake_item = STAKES.load(deps.storage, (&token_denom, stake_response.stake_id))?;
    let stake_amount = stake_item.token_amount;

    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;

    let mut response = Response::new();
    let attributes: Vec<Attribute> = vec![];
    let mut events: Vec<Event> = vec![];

    if stake_response.status == Status::Success {
        // update stake stats
        stake_stats.pending_stake -= Uint256::from(stake_amount);
        stake_stats.lp_token_amount += Uint256::from(stake_response.lp_token_amount);

        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        stake_item.action_stage = StakeActionStage::Executed;
        // todo: add attributes? events?

        // CW20 LP mint message
        let lp_mint_msg = create_cw20_mint_msg(
            &token_config.lp_token_address,
            &stake_item.user,
            stake_response.lp_token_amount,
        )
        .ok_or(ContractError::CustomError(
            "Can't create CW20 mint message".to_owned(),
        ))?;
        response = response.add_message(lp_mint_msg)
    } else {
        // update stake and user stats
        stake_stats.pending_stake -= Uint256::from(stake_amount);

        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        stake_item.action_stage = StakeActionStage::Failed;

        // todo: add attributes? events?

        // CW20 deposit token transfer message
        let cw20_transfer_msg = create_cw20_transfer_msg(
            &token_config.cw20_address,
            &stake_item.user,
            stake_item.token_amount,
        )
        .ok_or(ContractError::CustomError(
            "Can't create CW20 transfer message".to_owned(),
        ))?;
        response = response.add_message(cw20_transfer_msg)
        // response = response.add_message(BankMsg::Send {
        //     to_address: stake_item.user.to_string(),
        //     amount: vec![Coin {
        //         denom: token_denom.clone(),
        //         amount: stake_amount,
        //     }],
        // });
    }

    // update stake item
    STAKES.save(
        deps.storage,
        (&token_denom, stake_response.stake_id.clone()),
        &stake_item,
    )?;

    // decrease stake pending_count
    let mut stake_params = STAKE_PARAMS.load(deps.storage, &token_denom)?;
    stake_params.pending_count -= 1;
    STAKE_PARAMS.save(deps.storage, &token_denom, &stake_params)?;

    // handle reinit
    if stake_response.reinit_unstake_id != 0 {
        // get unstake amount
        let unstake_amount = token_amount
            .checked_sub(stake_amount)
            .map_err(|err| ContractError::Std(StdError::from(err)))?;

        let (reinit_wasm_msg, reinit_event) = handle_reinit(
            deps,
            &token_denom,
            &token_config.cw20_address,
            unstake_amount,
            &stake_response.reinit_unstake_id,
            stake_stats,
        )?;
        response = response.add_message(reinit_wasm_msg);
        events.push(reinit_event);
    }

    response = response.add_attributes(attributes).add_events(events);

    Ok(response)
}

fn ensure_stake_response_is_valid(
    token_amount: Uint128,
    _token_denom: &str,
    stake_response: &StakeResponseData,
) -> Result<(), ContractError> {
    if token_amount.is_zero() {
        if stake_response.reinit_unstake_id != 0 {
            return Err(ContractError::CustomError(
                "Stake response: reinit_unstake_id != 0, but message have no tokens".to_string(),
            ));
        }
        if stake_response.status == Status::Fail {
            return Err(ContractError::CustomError(
                "Fail stake response must have tokens in message".to_string(),
            ));
        }
    }
    if !token_amount.is_zero() {
        if stake_response.reinit_unstake_id == 0 && stake_response.status == Status::Success {
            return Err(ContractError::CustomError(
                "Stake response: reinit_unstake_id == 0 and status is Success, but message have tokens".to_string(),
            ));
        }
    }

    Ok(())

    // if info.funds.len() == 0 {
    //     if stake_response.reinit_unstake_id != 0 {
    //         return Err(ContractError::CustomError(
    //             "Stake response: reinit_unstake_id != 0, but message have no tokens".to_string(),
    //         ));
    //     }
    //     if stake_response.status == Status::Fail {
    //         return Err(ContractError::CustomError(
    //             "Fail stake response must have tokens in message".to_string(),
    //         ));
    //     }
    // }
    // if info.funds.len() == 1 {
    //     if stake_response.reinit_unstake_id == 0 && stake_response.status == Status::Success {
    //         return Err(ContractError::CustomError(
    //             "Stake response: reinit_unstake_id == 0 and status is Success, but message have tokens".to_string(),
    //         ));
    //     }
    //     let coin = info.funds.first().unwrap();
    //     if coin.denom != *token_denom {
    //         return Err(ContractError::InvalidToken {
    //             expected: token_denom.to_owned(),
    //             actual: coin.denom.clone(),
    //         });
    //     }
    // }
    // if info.funds.len() > 1 {
    //     return Err(ContractError::CustomError(
    //         "Stake response has too much coins in message".to_string(),
    //     ));
    // }
    //
    // Ok(())
}

pub fn try_handle_unstake_response(
    deps: DepsMut,
    _env: Env,
    // info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: &[u8],
    lpt_amount: Uint128,
) -> Result<Response, ContractError> {
    let (token_denom, token_config) =
        find_token_by_message_source(deps.as_ref(), &source_chain, &source_address)?;
    let unstake_response =
        decode_unstake_response_payload(payload).ok_or(ContractError::InvalidMessagePayload)?;

    ensure_unstake_response_is_valid(lpt_amount, &token_denom, &unstake_response)?;

    let mut unstake_item =
        UNSTAKES.load(deps.storage, (&token_denom, unstake_response.unstake_id))?;

    // todo: discuss it
    if unstake_item.action_stage != UnstakeActionStage::WaitingRegistration {
        return Err(ContractError::UnstakeRequestInvalidStage {
            symbol: token_config.symbol,
            unstake_id: unstake_response.unstake_id,
        });
    }
    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;

    let mut response = Response::new();
    let attributes: Vec<Attribute> = vec![];
    let mut events: Vec<Event> = vec![];

    if unstake_response.status == Status::Success {
        // update token stats
        stake_stats.lp_token_amount -= Uint256::from(unstake_item.lp_token_amount);
        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        // update action stage
        unstake_item.action_stage = UnstakeActionStage::Registered;

        // todo: burn LP tokens
    } else {
        // update token stats
        stake_stats.pending_unstake_lp_token_amount -= Uint256::from(unstake_item.lp_token_amount);
        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        // pop unstake and update unstake params
        UNSTAKES.remove(
            deps.storage,
            (&token_denom, unstake_response.unstake_id.clone()),
        );
        let mut unstake_params = UNSTAKE_PARAMS.load(deps.storage, &token_denom)?;
        unstake_params.pending_count -= 1;
        UNSTAKE_PARAMS.save(deps.storage, &token_denom, &unstake_params)?;

        // update action stage
        unstake_item.action_stage = UnstakeActionStage::Failed;

        let lp_mint_msg = create_cw20_mint_msg(
            &token_config.lp_token_address,
            &unstake_item.user,
            unstake_item.lp_token_amount,
        )
        .ok_or(ContractError::CustomError(
            "Can't create CW20 mint message".to_owned(),
        ))?;
        response = response.add_message(lp_mint_msg);
    }

    UNSTAKES.save(
        deps.storage,
        (&token_denom, unstake_response.unstake_id.clone()),
        &unstake_item,
    )?;

    // handle reinit
    if unstake_response.reinit_unstake_id != 0 {
        // get unstake amount

        let (reinit_wasm_msg, reinit_event) = handle_reinit(
            deps,
            &token_denom,
            &token_config.cw20_address,
            lpt_amount,
            &unstake_response.reinit_unstake_id,
            stake_stats,
        )?;
        response = response.add_message(reinit_wasm_msg);
        events.push(reinit_event);
    }

    response = response.add_attributes(attributes).add_events(events);

    Ok(response)
}

fn ensure_unstake_response_is_valid(
    lpt_amount: Uint128,
    _token_denom: &str,
    unstake_response: &UnstakeResponseData,
) -> Result<(), ContractError> {
    // assert message funds
    if lpt_amount.is_zero() && unstake_response.reinit_unstake_id != 0 {
        return Err(ContractError::CustomError(
            "Unstake response: reinit_unstake_id != 0, but message have no tokens".to_string(),
        ));
    }
    if !lpt_amount.is_zero() {
        if unstake_response.reinit_unstake_id == 0 {
            return Err(ContractError::CustomError(
                "Unstake response: reinit_unstake_id == 0, but message have tokens".to_string(),
            ));
        }
    }
    Ok(())

    // if info.funds.len() == 0 && unstake_response.reinit_unstake_id != 0 {
    //     return Err(ContractError::CustomError(
    //         "Unstake response: reinit_unstake_id != 0, but message have no tokens".to_string(),
    //     ));
    // }
    // if info.funds.len() == 1 {
    //     if unstake_response.reinit_unstake_id == 0 {
    //         return Err(ContractError::CustomError(
    //             "Unstake response: reinit_unstake_id == 0, but message have tokens".to_string(),
    //         ));
    //     }
    //     let coin = info.funds.first().unwrap();
    //     if coin.denom != *token_denom {
    //         return Err(ContractError::InvalidToken {
    //             expected: token_denom.to_owned(),
    //             actual: coin.denom.clone(),
    //         });
    //     }
    // }
    // if info.funds.len() > 1 {
    //     return Err(ContractError::CustomError(
    //         "Unstake response has too much coins in message".to_string(),
    //     ));
    // }
    // Ok(())
}

fn try_handle_reinit_response(
    deps: DepsMut,
    _env: Env,
    // info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: &[u8],
    token_amount: Uint128,
) -> Result<Response, ContractError> {
    if token_amount.is_zero() {
        return Err(ContractError::CustomError(
            "Reinit message must have one type of coins as funds".to_string(),
        ));
    }

    let (token_denom, token_config) =
        find_token_by_message_source(deps.as_ref(), &source_chain, &source_address)?;

    let reinit_response_data =
        decode_reinit_response_payload(payload).ok_or(ContractError::InvalidMessagePayload)?;

    let stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    let (mint_msg, event) = handle_reinit(
        deps,
        &token_denom,
        &token_config.cw20_address,
        token_amount,
        &reinit_response_data.reinit_unstake_id,
        stake_stats,
    )?;

    Ok(Response::new().add_message(mint_msg).add_event(event))
}

fn handle_reinit(
    deps: DepsMut,
    deposit_token_denom: &TokenDenom,
    deposit_token_address: &Addr,
    token_amount: Uint128,
    reinit_unstake_id: &u64,
    mut stake_stats: StakeStatsItem,
) -> Result<(WasmMsg, Event), ContractError> {
    let mut unstake_item = UNSTAKES.load(
        deps.storage,
        (&deposit_token_denom, reinit_unstake_id.clone()),
    )?;

    // return deposit + earnings to user
    let cw20_transfer_msg =
        create_cw20_transfer_msg(deposit_token_address, &unstake_item.user, token_amount).ok_or(
            ContractError::CustomError("Can't create CW20 transfer message".to_owned()),
        )?;

    // update unstake item
    unstake_item.action_stage = UnstakeActionStage::Executed;
    UNSTAKES.save(
        deps.storage,
        (&deposit_token_denom, reinit_unstake_id.clone()),
        &unstake_item,
    )?;

    // decrease unstake pending_count
    let unstake_params = UNSTAKE_PARAMS.load(deps.storage, &deposit_token_denom)?;
    UNSTAKE_PARAMS.save(
        deps.storage,
        &deposit_token_denom,
        &QueueParams {
            pending_count: unstake_params.pending_count - 1,
            next_id: unstake_params.next_id,
        },
    )?;

    stake_stats.pending_unstake_lp_token_amount -= Uint256::from(unstake_item.lp_token_amount);
    STAKE_STATS.save(deps.storage, &deposit_token_denom, &stake_stats)?;

    // todo: add message to burn LP tokens

    Ok((
        cw20_transfer_msg,
        Event::new("unstake_finished")
            .add_attribute("token", deposit_token_denom)
            .add_attribute("lp_amount", unstake_item.lp_token_amount)
            .add_attribute("token_amount", token_amount),
    ))
}

pub fn try_add_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_denom: String,
    cw20_address: Addr,
    is_stake_enabled: bool,
    is_unstake_enabled: bool,
    chain: String,
    symbol: String,
    name: String,
    evm_yield_contract: String,
    evm_address: String,
    lp_token_denom: String,
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

    let msg = to_json_binary(&LpInstantiateMsg {
        name,
        symbol: symbol.clone(),
        decimals: 6,
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

    let lp_token_address =
        calculate_token_address(deps.as_ref(), env, contract_config.lp_token_code_id, salt)?;

    TOKEN_CONFIG.save(
        deps.storage,
        &token_denom,
        &TokenConfig {
            cw20_address,
            is_stake_enabled,
            is_unstake_enabled,
            symbol,
            chain,
            evm_yield_contract,
            evm_address,
            lp_token_denom,
            lp_token_address,
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

pub fn try_update_token_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: String,
    config: TokenConfig,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    if !TOKEN_CONFIG.has(deps.storage, &token_denom) {
        return Err(ContractError::UnknownToken(token_denom.clone()));
    }
    TOKEN_CONFIG.save(deps.storage, &token_denom, &config)?;

    Ok(Response::default())
}

pub fn try_mint_lp_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Addr,
    lp_token_address: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    if !CONTRACT_CONFIG.load(deps.storage)?.is_mint_allowed {
        return Err(ContractError::MintIsNowAllowed);
    }

    let mint_msg = create_cw20_mint_msg(&lp_token_address, &recipient, amount).ok_or(
        ContractError::CustomError("Can't create CW20 mint message".to_owned()),
    )?;

    Ok(Response::new().add_message(mint_msg))
}

fn create_cw20_mint_msg(cw20_address: &Addr, recipient: &Addr, amount: Uint128) -> Option<WasmMsg> {
    let msg = to_json_binary(&Cw20ExecuteMsg::Mint {
        recipient: recipient.to_string(),
        amount,
    })
    .unwrap();

    Some(WasmMsg::Execute {
        contract_addr: cw20_address.to_string(),
        msg,
        funds: vec![],
    })
}

fn create_cw20_transfer_msg(
    cw20_address: &Addr,
    recipient: &Addr,
    amount: Uint128,
) -> Option<WasmMsg> {
    let msg = to_json_binary(&Cw20ExecuteMsg::Transfer {
        recipient: recipient.to_string(),
        amount,
    })
    .unwrap();

    Some(WasmMsg::Execute {
        contract_addr: cw20_address.to_string(),
        msg,
        funds: vec![],
    })
}

pub fn try_disallow_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    let mut contract_config = CONTRACT_CONFIG.load(deps.storage)?;
    contract_config.is_mint_allowed = false;
    CONTRACT_CONFIG.save(deps.storage, &contract_config)?;

    Ok(Response::new())
}

pub fn try_receive_cw_20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let action_msg: Cw20ActionMsg = from_json(msg.msg)
        .ok()
        .ok_or(ContractError::InvalidCw20Message)?;

    let user = deps.api.addr_validate(&msg.sender)?;

    match action_msg {
        Cw20ActionMsg::Stake {
            deposit_token_denom,
        } => {
            assert_cw20_deposit_token_address(deps.as_ref(), &deposit_token_denom, &info.sender)?;
            if msg.amount.is_zero() {
                return Err(ContractError::ZeroTokenAmount);
            }
            try_init_stake(deps, env, user, deposit_token_denom, msg.amount)
        }
        Cw20ActionMsg::Unstake {
            deposit_token_denom,
        } => {
            if msg.amount.is_zero() {
                return Err(ContractError::ZeroTokenAmount);
            }
            assert_cw20_lpt_address(deps.as_ref(), &deposit_token_denom, &info.sender)?;
            try_init_unstake(deps, env, user, deposit_token_denom, msg.amount)
        }
        Cw20ActionMsg::HandleResponse {
            deposit_token_denom,
            source_chain,
            source_address,
            payload,
        } => {
            assert_cw20_deposit_token_address(deps.as_ref(), &deposit_token_denom, &info.sender)?;
            try_handle_response(
                deps,
                env,
                user,
                source_chain,
                source_address,
                payload,
                msg.amount,
            )
        }
    }
}

fn assert_cw20_deposit_token_address(
    deps: Deps,
    token_denom: &TokenDenom,
    actual_cw20_address: &Addr,
) -> Result<(), ContractError> {
    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &token_denom)?
        .ok_or(ContractError::UnknownToken(token_denom.clone()))?;
    if token_config.cw20_address != actual_cw20_address {
        return Err(ContractError::MismatchCw20Token {
            actual: actual_cw20_address.to_string(),
            expected: token_config.cw20_address.to_string(),
        });
    }
    Ok(())
}

fn assert_cw20_lpt_address(
    deps: Deps,
    token_denom: &TokenDenom,
    actual_cw20_address: &Addr,
) -> Result<(), ContractError> {
    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &token_denom)?
        .ok_or(ContractError::UnknownToken(token_denom.clone()))?;
    if token_config.lp_token_address != actual_cw20_address {
        return Err(ContractError::MismatchCw20Token {
            actual: actual_cw20_address.to_string(),
            expected: token_config.cw20_address.to_string(),
        });
    }
    Ok(())
}
