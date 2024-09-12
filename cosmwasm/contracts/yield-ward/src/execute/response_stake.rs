use crate::encoding::decode_stake_response_payload;
use crate::execute::common::{create_bank_transfer_msg, create_cw20_mint_msg};
use crate::execute::reinit::handle_reinit;
use crate::helpers::find_token_by_message_source;
use crate::state::{STAKES, STAKE_PARAMS, STAKE_STATS};
use crate::types::{StakeActionStage, StakeResponseData, Status};
use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, StdError, Uint256};

pub fn try_handle_stake_response(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    source_chain: String,
    source_address: String,
    payload: &[u8],
) -> Result<Response, ContractError> {
    let stake_response =
        decode_stake_response_payload(payload).ok_or(ContractError::InvalidMessagePayload)?;

    let (token_denom, token_config) =
        find_token_by_message_source(deps.as_ref(), &source_chain, &source_address)?;

    ensure_stake_response_is_valid(&info, &token_denom, &stake_response)?;

    let mut stake_item = STAKES.load(deps.storage, (&token_denom, stake_response.stake_id))?;
    let stake_amount = stake_item.token_amount;

    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;

    let mut response = Response::new();

    if stake_response.status == Status::Success {
        // update stake stats
        stake_stats.pending_stake -= Uint256::from(stake_amount);
        stake_stats.lp_token_amount += Uint256::from(stake_response.lp_token_amount);

        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        stake_item.action_stage = StakeActionStage::Executed;
        stake_item.lp_token_amount = Some(stake_response.lp_token_amount);

        // CW20 LP mint message
        let lp_mint_msg = create_cw20_mint_msg(
            &token_config.lpt_address,
            &stake_item.user,
            stake_response.lp_token_amount,
        )?;

        response = response.add_message(lp_mint_msg).add_event(
            Event::new("stake_success")
                .add_attribute("stake_id", stake_response.stake_id.to_string())
                .add_attribute("lp_amount", stake_response.lp_token_amount)
                .add_attribute("token_amount", stake_amount),
        );
    } else {
        // update stake and user stats
        stake_stats.pending_stake -= Uint256::from(stake_amount);

        STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

        stake_item.action_stage = StakeActionStage::Failed;

        // return funds to user
        response = response
            .add_message(create_bank_transfer_msg(
                &stake_item.user,
                &token_denom,
                stake_amount,
            ))
            .add_event(
                Event::new("stake_failed")
                    .add_attribute("stake_id", stake_response.stake_id.to_string())
                    .add_attribute("token_amount", stake_amount),
            );
    }

    // update stake item
    STAKES.save(
        deps.storage,
        (&token_denom, stake_response.stake_id),
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
        let unstake_amount = match stake_response.status {
            Status::Success => coin.amount,
            Status::Fail => coin
                .amount
                .checked_sub(stake_amount)
                .map_err(|err| ContractError::Std(StdError::from(err)))?,
        };

        let (bank_transfer_msg, reinit_event) = handle_reinit(
            deps,
            &token_denom,
            unstake_amount,
            stake_response.reinit_unstake_id,
            stake_stats,
        )?;

        response = response
            .add_message(bank_transfer_msg)
            .add_event(reinit_event);
    }

    Ok(response)
}

fn ensure_stake_response_is_valid(
    info: &MessageInfo,
    token_denom: &str,
    stake_response: &StakeResponseData,
) -> Result<(), ContractError> {
    match info.funds.len() {
        0 => {
            if stake_response.reinit_unstake_id != 0 {
                return Err(ContractError::CustomError(
                    "Stake response: reinit_unstake_id != 0, but message have no tokens"
                        .to_string(),
                ));
            }
            if stake_response.status == Status::Fail {
                return Err(ContractError::CustomError(
                    "Fail stake response must have tokens in message".to_string(),
                ));
            }
        }
        1 => {
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
        _ => {
            return Err(ContractError::CustomError(
                "Stake response has too much coins in message".to_string(),
            ))
        }
    }

    Ok(())
}
