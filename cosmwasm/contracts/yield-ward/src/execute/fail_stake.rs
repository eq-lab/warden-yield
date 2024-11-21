use crate::helpers::assert_msg_sender_is_admin;
use crate::state::{STAKES, STAKE_STATS};
use crate::types::{StakeActionStage, TokenDenom};
use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Uint256};

pub fn try_fail_stake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: TokenDenom,
    stake_id: u64,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    let mut stake_item = STAKES.load(deps.storage, (&token_denom, stake_id))?;

    if stake_item.action_stage != StakeActionStage::Execution {
        return Err(ContractError::CustomError(
            "Fail stake is not allowed on current stage".to_string(),
        ));
    }

    stake_item.action_stage = StakeActionStage::Fail;

    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    stake_stats.pending_stake -= Uint256::from(stake_item.token_amount);

    STAKES.save(deps.storage, (&token_denom, stake_id), &stake_item)?;
    STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

    Ok(Response::new().add_event(
        Event::new("fail_stake")
            .add_attribute("token_denom", token_denom)
            .add_attribute("stake_id", stake_id.to_string())
            .add_attribute("user", stake_item.user.to_string()),
    ))
}
