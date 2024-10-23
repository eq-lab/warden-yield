use crate::execute::common::create_cw20_transfer_msg;
use crate::helpers::assert_msg_sender_is_admin;
use crate::ContractError;
use cosmwasm_std::QueryRequest::Wasm;
use cosmwasm_std::{to_json_binary, Addr, DepsMut, Env, Event, MessageInfo, Response, WasmQuery};
use cw20::{BalanceResponse, Cw20QueryMsg};

pub fn try_withdraw_cw20_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_address: Addr,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    let balance: BalanceResponse = deps.querier.query(&Wasm(WasmQuery::Smart {
        contract_addr: token_address.to_string(),
        msg: to_json_binary(&Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        })
        .map_err(|e| ContractError::Std(e))?,
    }))?;

    if balance.balance.is_zero() {
        return Err(ContractError::ZeroTokenAmount);
    }

    let msg = create_cw20_transfer_msg(&token_address, &info.sender, balance.balance)?;

    Ok(Response::new()
        .add_event(
            Event::new("withdraw_cw20_token")
                .add_attribute("token_address", token_address)
                .add_attribute("recipient", info.sender)
                .add_attribute("amount", balance.balance),
        )
        .add_message(msg))
}
