use crate::encoding::encode_stake_payload;
use crate::execute::axelar_messaging::send_message_evm;
use crate::state::{QueueParams, StakeItem, STAKES, STAKE_PARAMS, STAKE_STATS, TOKEN_CONFIG};
use crate::types::StakeActionStage;
use crate::ContractError;
use cosmwasm_std::{to_hex, DepsMut, Env, Event, MessageInfo, Response, Uint128, Uint256};

pub fn try_init_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fee_amount: Uint128,
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::CustomError(
            "Init stake message must have one type of coins as funds".to_string(),
        ));
    }
    let fund = info.funds.first().unwrap();

    if fee_amount >= fund.amount {
        return Err(ContractError::CustomError(
            "Fee amount should be less than attached amount".into(),
        ));
    }

    let stake_amount = fund.amount - fee_amount;

    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &fund.denom)?
        .ok_or(ContractError::UnknownToken(fund.denom.clone()))?;

    // check is staking enabled
    if !token_config.is_stake_enabled {
        return Err(ContractError::StakeDisabled(token_config.lpt_symbol));
    }

    let stake_params = STAKE_PARAMS.load(deps.storage, &fund.denom)?;
    let stake_id = stake_params.next_id;

    // push to stakes map
    STAKES.save(
        deps.storage,
        (&fund.denom, stake_id),
        &StakeItem {
            user: info.sender.clone(),
            token_amount: stake_amount,
            action_stage: StakeActionStage::WaitingExecution,
            lp_token_amount: None,
        },
    )?;

    // increment stake next_id
    STAKE_PARAMS.save(
        deps.storage,
        &fund.denom,
        &QueueParams {
            pending_count: stake_params.pending_count + 1,
            next_id: stake_id + 1,
        },
    )?;

    // update stake stats
    let mut stake_stats = STAKE_STATS.load(deps.storage, &fund.denom)?;
    stake_stats.pending_stake += Uint256::from(stake_amount);
    STAKE_STATS.save(deps.storage, &fund.denom, &stake_stats)?;

    let stake_payload = encode_stake_payload(stake_id);
    let payload_hex_str = to_hex(&stake_payload);

    let response = send_message_evm(
        deps.as_ref(),
        env,
        fund,
        &token_config,
        stake_payload,
        fee_amount,
    )?;

    Ok(response.add_event(
        Event::new("stake")
            .add_attribute("stake_id", stake_id.to_string())
            .add_attribute("sender", info.sender)
            .add_attribute("chain", token_config.chain)
            .add_attribute("yield_contract", token_config.evm_yield_contract)
            .add_attribute("token_amount", stake_amount)
            .add_attribute("fee", fee_amount)
            .add_attribute("payload", "0x".to_owned() + &payload_hex_str),
    ))
}
