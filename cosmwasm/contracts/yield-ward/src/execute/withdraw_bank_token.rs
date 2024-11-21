use crate::execute::common::create_bank_transfer_msg;
use crate::helpers::assert_msg_sender_is_admin;
use crate::types::TokenDenom;
use crate::ContractError;
use cosmwasm_std::BankQuery::Balance;
use cosmwasm_std::QueryRequest::Bank;
use cosmwasm_std::{BalanceResponse, DepsMut, Env, Event, MessageInfo, Response};

pub fn try_withdraw_bank_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: TokenDenom,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    let balance: BalanceResponse = deps.querier.query(&Bank(Balance {
        address: info.sender.to_string(),
        denom: token_denom.to_string(),
    }))?;

    if balance.amount.amount.is_zero() {
        return Err(ContractError::ZeroTokenAmount);
    }

    let msg = create_bank_transfer_msg(&info.sender, &token_denom, balance.amount.amount);

    Ok(Response::new()
        .add_event(
            Event::new("withdraw_bank_token")
                .add_attribute("token_denom", token_denom)
                .add_attribute("recipient", info.sender)
                .add_attribute("amount", balance.amount.amount),
        )
        .add_message(msg))
}
