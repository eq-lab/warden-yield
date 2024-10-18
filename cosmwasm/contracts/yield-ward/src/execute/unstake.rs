use crate::encoding::encode_unstake_payload;
use crate::execute::axelar_messaging::send_message_evm;
use crate::state::{UnstakeItem, STAKE_STATS, TOKEN_CONFIG, UNSTAKES, UNSTAKE_PARAMS};
use crate::types::{TokenDenom, UnstakeActionStage};
use crate::ContractError;
use cosmwasm_std::{to_hex, Addr, DepsMut, Env, Event, MessageInfo, Response, Uint128, Uint256};

pub fn try_init_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user: Addr,
    token_denom: TokenDenom,
    lpt_amount: Uint128,
) -> Result<Response, ContractError> {
    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &token_denom)?
        .ok_or(ContractError::UnknownToken(token_denom.clone()))?;

    if info.funds.len() != 1 {
        return Err(ContractError::CustomError(
            "Wrong number of tokens attached to unstake call".to_string(),
        ));
    }

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
            user: user.clone(),
            lp_token_amount: lpt_amount,
            action_stage: UnstakeActionStage::Execution,
            token_amount: None,
        },
    )?;

    // update stake stats
    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    stake_stats.pending_unstake_lp_token_amount += Uint256::from(lpt_amount);
    STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

    let fund = info.funds.first().unwrap();
    let unstake_payload = encode_unstake_payload(unstake_id, &lpt_amount);
    let payload_hex_str = to_hex(&unstake_payload);

    let response = send_message_evm(
        deps.as_ref(),
        env,
        fund,
        &token_config,
        unstake_payload,
        fund.amount,
    )?;

    Ok(response.add_event(
        Event::new("unstake")
            .add_attribute("unstake_id", unstake_id.to_string())
            .add_attribute("sender", user)
            .add_attribute("chain", token_config.chain)
            .add_attribute("yield_contract", token_config.evm_yield_contract)
            .add_attribute("lpt_amount", lpt_amount)
            .add_attribute("payload", "0x".to_owned() + &payload_hex_str),
    ))
}
