use crate::encoding::encode_stake_payload;
use crate::state::{QueueParams, StakeItem, STAKES, STAKE_PARAMS, STAKE_STATS, TOKEN_CONFIG};
use crate::types::{StakeActionStage, TokenDenom};
use crate::ContractError;
use cosmwasm_std::{to_hex, Addr, DepsMut, Env, Event, Response, Uint128, Uint256};

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
        return Err(ContractError::StakeDisabled(token_config.lpt_symbol));
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
            lp_token_amount: None,
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
            .add_attribute("stake_id", stake_id.to_string())
            .add_attribute("token_symbol", token_config.deposit_token_symbol)
            .add_attribute("evm_yield_contract", token_config.evm_yield_contract)
            .add_attribute("dest_chain", token_config.chain)
            .add_attribute("token_amount", token_amount)
            .add_attribute("payload", "0x".to_owned() + &payload_hex_str),
    ))
}
