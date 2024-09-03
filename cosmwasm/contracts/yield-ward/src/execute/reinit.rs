use crate::encoding::{decode_reinit_response_payload, encode_reinit_payload};
use crate::execute::common::create_cw20_transfer_msg;
use crate::helpers::find_token_by_message_source;
use crate::state::{
    QueueParams, StakeStatsItem, STAKE_STATS, TOKEN_CONFIG, UNSTAKES, UNSTAKE_PARAMS,
};
use crate::types::{TokenDenom, UnstakeActionStage};
use crate::ContractError;
use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Response, Uint128, Uint256, WasmMsg};

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

pub fn handle_reinit(
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

pub fn try_handle_reinit_response(
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
