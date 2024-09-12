use crate::encoding::{decode_reinit_response_payload, encode_reinit_payload};
use crate::execute::common::create_bank_transfer_msg;
use crate::helpers::find_token_by_message_source;
use crate::state::{
    QueueParams, StakeStatsItem, STAKE_STATS, TOKEN_CONFIG, UNSTAKES, UNSTAKE_PARAMS,
};
use crate::types::{TokenDenom, UnstakeActionStage};
use crate::ContractError;
use cosmwasm_std::{BankMsg, DepsMut, Env, Event, MessageInfo, Response, Uint128, Uint256};

pub fn try_reinit(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    token_denom: TokenDenom,
) -> Result<Response, ContractError> {
    let _token_config = TOKEN_CONFIG.load(deps.storage, &token_denom)?;

    let _reinit_payload = encode_reinit_payload();

    // todo: send reinit message to Axelar

    Ok(Response::new())
}

pub fn handle_reinit(
    deps: DepsMut,
    deposit_token_denom: &TokenDenom,
    token_amount: Uint128,
    reinit_unstake_id: &u64,
    mut stake_stats: StakeStatsItem,
) -> Result<(BankMsg, Event), ContractError> {
    let mut unstake_item = UNSTAKES.load(
        deps.storage,
        (&deposit_token_denom, reinit_unstake_id.clone()),
    )?;

    // return deposit + earnings to user
    let bank_transfer_msg =
        create_bank_transfer_msg(&unstake_item.user, deposit_token_denom, token_amount);

    // update unstake item
    unstake_item.action_stage = UnstakeActionStage::Executed;
    unstake_item.token_amount = match unstake_item.token_amount {
        Some(amount) => Some(amount + token_amount),
        None => Some(token_amount),
    };

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

    Ok((
        bank_transfer_msg,
        Event::new("unstake_finished")
            .add_attribute("unstake_id", reinit_unstake_id.to_string())
            .add_attribute("token", deposit_token_denom)
            .add_attribute("lp_amount", unstake_item.lp_token_amount)
            .add_attribute("token_amount", token_amount)
            .add_attribute("total_token_amount", unstake_item.token_amount.unwrap()),
    ))
}

pub fn try_handle_reinit_response(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: &[u8],
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 || info.funds.first().unwrap().amount.is_zero() {
        return Err(ContractError::CustomError(
            "Reinit message must have one type of coins as funds".to_string(),
        ));
    }

    let coin = info.funds.first().unwrap();
    let (token_denom, _token_config) =
        find_token_by_message_source(deps.as_ref(), &source_chain, &source_address)?;

    if token_denom != coin.denom {
        return Err(ContractError::InvalidToken {
            actual: coin.denom.to_string(),
            expected: token_denom,
        });
    }

    let reinit_response_data =
        decode_reinit_response_payload(payload).ok_or(ContractError::InvalidMessagePayload)?;

    let stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    let (bank_transfer_msg, event) = handle_reinit(
        deps,
        &token_denom,
        coin.amount,
        &reinit_response_data.reinit_unstake_id,
        stake_stats,
    )?;

    Ok(Response::new()
        .add_message(bank_transfer_msg)
        .add_event(event))
}
