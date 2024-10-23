use crate::helpers::assert_msg_sender_is_admin;
use crate::state::{STAKE_STATS, UNSTAKES};
use crate::types::{TokenDenom, UnstakeActionStage};
use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Uint256};

pub fn try_fail_unstake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: TokenDenom,
    unstake_id: u64,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    let mut unstake_item = UNSTAKES.load(deps.storage, (&token_denom, unstake_id))?;

    match unstake_item.action_stage {
        UnstakeActionStage::Execution | UnstakeActionStage::Queued => {
            unstake_item.action_stage = UnstakeActionStage::Fail;
        }
        _ => {
            return Err(ContractError::CustomError(
                "Fail unstake is not allowed on current stage".to_string(),
            ));
        }
    };

    let mut stake_stats = STAKE_STATS.load(deps.storage, &token_denom)?;
    stake_stats.pending_unstake_lp_token_amount -= Uint256::from(unstake_item.lp_token_amount);

    UNSTAKES.save(deps.storage, (&token_denom, unstake_id), &unstake_item)?;
    STAKE_STATS.save(deps.storage, &token_denom, &stake_stats)?;

    Ok(Response::new().add_event(
        Event::new("fail_unstake")
            .add_attribute("token_denom", token_denom)
            .add_attribute("unstake_id", unstake_id.to_string())
            .add_attribute("user", unstake_item.user.to_string()),
    ))
}
