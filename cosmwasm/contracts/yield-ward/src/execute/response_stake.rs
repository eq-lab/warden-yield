use crate::encoding::decode_stake_response_payload;
use crate::execute::common::{create_cw20_mint_msg, create_cw20_transfer_msg};
use crate::execute::reinit::handle_reinit;
use crate::helpers::find_token_by_message_source;
use crate::state::{STAKES, STAKE_PARAMS, STAKE_STATS};
use crate::types::{StakeActionStage, StakeResponseData, Status};
use crate::ContractError;
use cosmwasm_std::{Attribute, DepsMut, Env, Event, Response, StdError, Uint128, Uint256};

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

        // CW20 LP mint message
        let lp_mint_msg = create_cw20_mint_msg(
            &token_config.lpt_address,
            &stake_item.user,
            stake_response.lp_token_amount,
        )
        .ok_or(ContractError::CustomError(
            "Can't create CW20 mint message".to_owned(),
        ))?;
        response = response.add_message(lp_mint_msg)

        // todo: add event
    } else {
        // update stake and user stats
        stake_stats.pending_stake -= Uint256::from(stake_amount);

        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        stake_item.action_stage = StakeActionStage::Failed;

        // todo: add event

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
