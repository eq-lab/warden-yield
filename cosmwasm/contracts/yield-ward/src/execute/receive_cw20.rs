use crate::execute::unstake::try_init_unstake;
use crate::helpers::find_deposit_token_denom_by_lpt_address;
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
    let action_msg: Cw20ActionMsg =
        from_json(msg.msg).map_err(|_| ContractError::InvalidCw20Message)?;

    let user = deps.api.addr_validate(&msg.sender)?;

    match action_msg {
        Cw20ActionMsg::Unstake => {
            if msg.amount.is_zero() {
                return Err(ContractError::ZeroTokenAmount);
            }
            let deposit_token_denom =
                find_deposit_token_denom_by_lpt_address(deps.as_ref(), &info.sender)?;
            try_init_unstake(deps, env, user, deposit_token_denom, msg.amount)
        }
    }
}
