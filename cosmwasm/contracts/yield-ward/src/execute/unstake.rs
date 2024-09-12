use crate::encoding::encode_unstake_payload;
use crate::state::{UnstakeItem, STAKE_STATS, TOKEN_CONFIG, UNSTAKES, UNSTAKE_PARAMS};
use crate::types::{TokenDenom, UnstakeActionStage};
use crate::ContractError;
use cosmwasm_std::{to_hex, Addr, DepsMut, Env, Event, Response, Uint128, Uint256};

pub fn try_init_unstake(
    deps: DepsMut,
    _env: Env,
    user: Addr,
    token_denom: TokenDenom,
    lpt_amount: Uint128,
) -> Result<Response, ContractError> {
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
            user: user.clone(),
            lp_token_amount: lpt_amount,
            action_stage: UnstakeActionStage::WaitingRegistration,
            token_amount: None,
        },
    )?;

    // update stake stats
    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    stake_stats.pending_unstake_lp_token_amount += Uint256::from(lpt_amount);
    STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

    let unstake_payload = encode_unstake_payload(&unstake_id, &lpt_amount);
    // todo: send message to Axelar

    let payload_hex_str = to_hex(unstake_payload);

    Ok(Response::new().add_event(
        Event::new("unstake")
            .add_attribute("unstake_id", unstake_id.to_string())
            .add_attribute("sender", user)
            .add_attribute("token_symbol", token_config.deposit_token_symbol)
            .add_attribute("evm_yield_contract", token_config.evm_yield_contract)
            .add_attribute("dest_chain", token_config.chain)
            .add_attribute("lpt_amount", lpt_amount)
            .add_attribute("payload", "0x".to_owned() + &payload_hex_str),
    ))
}
