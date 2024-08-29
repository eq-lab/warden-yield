use crate::encoding::decode_unstake_response_payload;
use crate::execute::common::create_cw20_transfer_msg;
use crate::execute::reinit::handle_reinit;
use crate::helpers::find_token_by_message_source;
use crate::state::{STAKE_STATS, UNSTAKES, UNSTAKE_PARAMS};
use crate::types::{Status, UnstakeActionStage, UnstakeResponseData};
use crate::ContractError;
use cosmwasm_std::{Attribute, DepsMut, Env, Event, Response, Uint128, Uint256};

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
            symbol: token_config.deposit_token_symbol,
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

        let lp_mint_msg = create_cw20_transfer_msg(
            &token_config.lpt_address,
            &unstake_item.user,
            unstake_item.lp_token_amount,
        )
        .ok_or(ContractError::CustomError(
            "Can't create CW20 transfer message".to_owned(),
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
