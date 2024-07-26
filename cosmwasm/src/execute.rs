use crate::encoding::{
    decode_payload_action_type, decode_reinit_response_payload, decode_stake_response_payload,
    decode_unstake_response_payload, encode_reinit_payload, encode_stake_payload,
    encode_unstake_payload,
};
use crate::helpers::{
    assert_msg_sender_is_admin, assert_msg_sender_is_axelar, find_token_by_lp_token_denom,
    find_token_by_message_source,
};
use crate::state::{
    QueueParams, StakeItem, StakeStatsItem, UnstakeItem, STAKES, STAKE_PARAMS, STAKE_STATS,
    TOKEN_CONFIG, UNSTAKES, UNSTAKE_PARAMS,
};
use crate::types::{
    ActionType, StakeActionStage, StakeResponseData, Status, TokenConfig, TokenDenom,
    UnstakeActionStage, UnstakeResponseData,
};
use crate::ContractError;
use cosmwasm_std::{
    Attribute, BankMsg, Binary, Coin, DepsMut, Env, Event, MessageInfo, Response, StdError, Uint256,
};

pub fn try_init_stake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::CustomError(
            "Init stake message must have one type of coins as funds".to_string(),
        ));
    }
    let coin = info.funds.first().unwrap();
    if coin.amount.is_zero() {
        return Err(ContractError::ZeroTokenAmount);
    }
    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &coin.denom)?
        .ok_or(ContractError::UnknownToken(coin.denom.clone()))?;

    // check is staking enabled
    if !token_config.is_stake_enabled {
        return Err(ContractError::StakeDisabled(token_config.symbol));
    }

    let stake_params = STAKE_PARAMS.load(deps.storage, &coin.denom)?;
    let stake_id = stake_params.next_id;

    // push to stakes map
    STAKES.save(
        deps.storage,
        (&coin.denom, stake_id),
        &StakeItem {
            user: info.sender,
            token_amount: coin.amount,
            action_stage: StakeActionStage::WaitingExecution,
        },
    )?;

    // increment stake next_id
    STAKE_PARAMS.save(
        deps.storage,
        &coin.denom,
        &QueueParams {
            pending_count: stake_params.pending_count + 1,
            next_id: stake_id + 1,
        },
    )?;

    // update stake stats
    let mut stake_stats = STAKE_STATS.load(deps.storage, &coin.denom)?;
    stake_stats.pending_stake += Uint256::from(coin.amount);
    STAKE_STATS.save(deps.storage, &coin.denom, &stake_stats)?;

    let _payload = encode_stake_payload(&stake_id);
    // todo: send tokens to Axelar

    Ok(Response::default())
}

pub fn try_init_unstake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::CustomError(
            "Init unstake message must have one type of coins as funds".to_string(),
        ));
    }

    let coin = info.funds.first().unwrap();

    let (token_denom, _token_config) = find_token_by_lp_token_denom(deps.as_ref(), &coin.denom)?;

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
            user: info.sender,
            lp_token_amount: coin.amount,
            action_stage: UnstakeActionStage::WaitingRegistration,
        },
    )?;

    // update stake stats
    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    stake_stats.pending_unstake_lp_token_amount += Uint256::from(coin.amount);
    STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

    let _unstake_payload = encode_unstake_payload(&unstake_id, &coin.amount);
    // todo: send message to Axelar

    Ok(Response::default())
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
    info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: Binary,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_axelar(deps.as_ref(), &info)?;

    let action_type =
        decode_payload_action_type(&payload).ok_or(ContractError::InvalidActionType)?;

    // skip ActionId first byte
    let payload = &payload[1..];
    match action_type {
        ActionType::Stake => {
            try_handle_stake_response(deps, env, info, source_chain, source_address, payload)
        }
        ActionType::Unstake => {
            try_handle_unstake_response(deps, env, info, source_chain, source_address, payload)
        }
        ActionType::Reinit => {
            try_handle_reinit_response(deps, env, info, source_chain, source_address, payload)
        }
    }
}

pub fn try_handle_stake_response(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: &[u8],
) -> Result<Response, ContractError> {
    assert_msg_sender_is_axelar(deps.as_ref(), &info)?;
    let stake_response =
        decode_stake_response_payload(payload).ok_or(ContractError::InvalidMessagePayload)?;

    let (token_denom, _) =
        find_token_by_message_source(deps.as_ref(), &source_chain, &source_address)?;

    ensure_stake_response_is_valid(&info, &token_denom, &stake_response)?;

    let mut stake_item = STAKES.load(deps.storage, (&token_denom, stake_response.stake_id))?;
    let stake_amount = stake_item.token_amount;

    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;

    let mut messages: Vec<BankMsg> = vec![];
    let attributes: Vec<Attribute> = vec![];
    let mut events: Vec<Event> = vec![];

    if stake_response.status == Status::Success {
        // update stake stats
        stake_stats.pending_stake -= Uint256::from(stake_amount);
        stake_stats.lp_token_amount += Uint256::from(stake_response.lp_token_amount);

        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        stake_item.action_stage = StakeActionStage::Executed;
        // todo: add attributes? events?
        // todo: create message to mint LP tokens
    } else {
        let coin = info.funds.first().unwrap();
        if coin.denom != *token_denom {
            return Err(ContractError::InvalidToken {
                expected: token_denom.clone(),
                actual: coin.denom.clone(),
            });
        }

        // update stake and user stats
        stake_stats.pending_stake -= Uint256::from(stake_amount);

        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        stake_item.action_stage = StakeActionStage::Failed;

        // todo: add attributes? events?
        messages.push(BankMsg::Send {
            to_address: stake_item.user.to_string(),
            amount: vec![Coin {
                denom: token_denom.clone(),
                amount: stake_amount,
            }],
        });
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
        let coin = info.funds.first().unwrap();
        // get unstake amount
        let unstake_amount = coin
            .amount
            .checked_sub(stake_amount)
            .map_err(|err| ContractError::Std(StdError::from(err)))?;

        let (reinit_bank_msg, reinit_event) = handle_reinit(
            deps,
            Coin {
                denom: token_denom,
                amount: unstake_amount,
            },
            &stake_response.reinit_unstake_id,
            stake_stats,
        )?;
        messages.push(reinit_bank_msg);
        events.push(reinit_event);
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(attributes)
        .add_events(events))
}

fn ensure_stake_response_is_valid(
    info: &MessageInfo,
    token_denom: &str,
    stake_response: &StakeResponseData,
) -> Result<(), ContractError> {
    if info.funds.len() == 0 {
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
    if info.funds.len() == 1 {
        if stake_response.reinit_unstake_id == 0 && stake_response.status == Status::Success {
            return Err(ContractError::CustomError(
                "Stake response: reinit_unstake_id == 0 and status is Success, but message have tokens".to_string(),
            ));
        }
        let coin = info.funds.first().unwrap();
        if coin.denom != *token_denom {
            return Err(ContractError::InvalidToken {
                expected: token_denom.to_owned(),
                actual: coin.denom.clone(),
            });
        }
    }
    if info.funds.len() > 1 {
        return Err(ContractError::CustomError(
            "Stake response has too much coins in message".to_string(),
        ));
    }

    Ok(())
}
pub fn try_handle_unstake_response(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: &[u8],
) -> Result<Response, ContractError> {
    assert_msg_sender_is_axelar(deps.as_ref(), &info)?;

    let (token_denom, token_config) =
        find_token_by_message_source(deps.as_ref(), &source_chain, &source_address)?;
    let unstake_response =
        decode_unstake_response_payload(payload).ok_or(ContractError::InvalidMessagePayload)?;

    ensure_unstake_response_is_valid(&info, &token_denom, &unstake_response)?;

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

    let mut messages: Vec<BankMsg> = vec![];
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

        // todo: return LP tokens to user
    }

    UNSTAKES.save(
        deps.storage,
        (&token_denom, unstake_response.unstake_id.clone()),
        &unstake_item,
    )?;

    // handle reinit
    if unstake_response.reinit_unstake_id != 0 {
        let coin = info.funds.first().unwrap();
        // get unstake amount
        let unstake_amount = coin.amount;

        let (reinit_bank_msg, reinit_event) = handle_reinit(
            deps,
            Coin {
                denom: token_denom,
                amount: unstake_amount,
            },
            &unstake_response.reinit_unstake_id,
            stake_stats,
        )?;
        messages.push(reinit_bank_msg);
        events.push(reinit_event);
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(attributes)
        .add_events(events))
}

fn ensure_unstake_response_is_valid(
    info: &MessageInfo,
    token_denom: &str,
    unstake_response: &UnstakeResponseData,
) -> Result<(), ContractError> {
    // assert message funds
    if info.funds.len() == 0 && unstake_response.reinit_unstake_id != 0 {
        return Err(ContractError::CustomError(
            "Unstake response: reinit_unstake_id != 0, but message have no tokens".to_string(),
        ));
    }
    if info.funds.len() == 1 {
        if unstake_response.reinit_unstake_id == 0 {
            return Err(ContractError::CustomError(
                "Unstake response: reinit_unstake_id == 0, but message have tokens".to_string(),
            ));
        }
        let coin = info.funds.first().unwrap();
        if coin.denom != *token_denom {
            return Err(ContractError::InvalidToken {
                expected: token_denom.to_owned(),
                actual: coin.denom.clone(),
            });
        }
    }
    if info.funds.len() > 1 {
        return Err(ContractError::CustomError(
            "Unstake response has too much coins in message".to_string(),
        ));
    }
    Ok(())
}

fn try_handle_reinit_response(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: &[u8],
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::CustomError(
            "Reinit message must have one type of coins as funds".to_string(),
        ));
    }

    let (token_denom, _token_config) =
        find_token_by_message_source(deps.as_ref(), &source_chain, &source_address)?;

    let coin = info.funds.first().unwrap();
    if coin.denom != token_denom {
        return Err(ContractError::InvalidToken {
            actual: coin.denom.clone(),
            expected: token_denom,
        });
    }

    let reinit_response_data =
        decode_reinit_response_payload(payload).ok_or(ContractError::InvalidMessagePayload)?;

    let stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    let (bank_msg, event) = handle_reinit(
        deps,
        coin.clone(),
        &reinit_response_data.reinit_unstake_id,
        stake_stats,
    )?;

    Ok(Response::new().add_message(bank_msg).add_event(event))
}

fn handle_reinit(
    deps: DepsMut,
    coin: Coin,
    reinit_unstake_id: &u64,
    mut stake_stats: StakeStatsItem,
) -> Result<(BankMsg, Event), ContractError> {
    let mut unstake_item = UNSTAKES.load(deps.storage, (&coin.denom, reinit_unstake_id.clone()))?;

    // return deposit + earnings to user
    let message = BankMsg::Send {
        to_address: unstake_item.user.to_string(),
        amount: vec![coin.clone()],
    };

    // update unstake item
    unstake_item.action_stage = UnstakeActionStage::Executed;
    UNSTAKES.save(
        deps.storage,
        (&coin.denom, reinit_unstake_id.clone()),
        &unstake_item,
    )?;

    // decrease unstake pending_count
    let unstake_params = UNSTAKE_PARAMS.load(deps.storage, &coin.denom)?;
    UNSTAKE_PARAMS.save(
        deps.storage,
        &coin.denom,
        &QueueParams {
            pending_count: unstake_params.pending_count - 1,
            next_id: unstake_params.next_id,
        },
    )?;

    stake_stats.pending_unstake_lp_token_amount -= Uint256::from(unstake_item.lp_token_amount);
    STAKE_STATS.save(deps.storage, &coin.denom, &stake_stats)?;

    // todo: add message to burn LP tokens

    Ok((
        message,
        Event::new("unstake_finished")
            .add_attribute("token", coin.denom)
            .add_attribute("lp_amount", unstake_item.lp_token_amount)
            .add_attribute("token_amount", coin.amount),
    ))
}

pub fn try_add_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: String,
    config: TokenConfig,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    if TOKEN_CONFIG.has(deps.storage, &token_denom) {
        return Err(ContractError::TokenAlreadyExist(token_denom));
    }

    TOKEN_CONFIG.save(deps.storage, &token_denom, &config)?;

    STAKE_STATS.save(deps.storage, &token_denom, &StakeStatsItem::default())?;

    // todo: instantiate LP token
    Ok(Response::default())
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
