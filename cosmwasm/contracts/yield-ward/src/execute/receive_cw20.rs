use crate::execute::common::{assert_cw20_deposit_token_address, assert_cw20_lpt_address};
use crate::execute::response::try_handle_response;
use crate::execute::stake::try_init_stake;
use crate::execute::unstake::try_init_unstake;
use crate::msg::Cw20ActionMsg;
use crate::ContractError;
use cosmwasm_std::{from_json, DepsMut, Env, MessageInfo, Response};
use cw20::Cw20ReceiveMsg;

pub fn try_receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let action_msg: Cw20ActionMsg = from_json(msg.msg)
        .ok()
        .ok_or(ContractError::InvalidCw20Message)?;

    let user = deps.api.addr_validate(&msg.sender)?;

    match action_msg {
        Cw20ActionMsg::Stake {
            deposit_token_denom,
        } => {
            assert_cw20_deposit_token_address(deps.as_ref(), &deposit_token_denom, &info.sender)?;
            if msg.amount.is_zero() {
                return Err(ContractError::ZeroTokenAmount);
            }
            try_init_stake(deps, env, user, deposit_token_denom, msg.amount)
        }
        Cw20ActionMsg::Unstake {
            deposit_token_denom,
        } => {
            if msg.amount.is_zero() {
                return Err(ContractError::ZeroTokenAmount);
            }
            assert_cw20_lpt_address(deps.as_ref(), &deposit_token_denom, &info.sender)?;
            try_init_unstake(deps, env, user, deposit_token_denom, msg.amount)
        }
        Cw20ActionMsg::HandleResponse {
            deposit_token_denom,
            source_chain,
            source_address,
            payload,
        } => {
            assert_cw20_deposit_token_address(deps.as_ref(), &deposit_token_denom, &info.sender)?;
            try_handle_response(
                deps,
                env,
                user,
                source_chain,
                source_address,
                payload,
                msg.amount,
            )
        }
    }
}
