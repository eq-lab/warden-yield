use crate::types::TokenDenom;
use cosmwasm_std::{to_json_binary, Addr, BankMsg, Coin, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

pub fn create_cw20_mint_msg(
    cw20_address: &Addr,
    recipient: &Addr,
    amount: Uint128,
) -> Option<WasmMsg> {
    let msg = to_json_binary(&Cw20ExecuteMsg::Mint {
        recipient: recipient.to_string(),
        amount,
    })
    .ok()?;

    Some(WasmMsg::Execute {
        contract_addr: cw20_address.to_string(),
        msg,
        funds: vec![],
    })
}

pub fn create_cw20_burn_msg(cw20_address: &Addr, amount: Uint128) -> Option<WasmMsg> {
    let msg = to_json_binary(&Cw20ExecuteMsg::Burn { amount }).ok()?;

    Some(WasmMsg::Execute {
        contract_addr: cw20_address.to_string(),
        msg,
        funds: vec![],
    })
}

pub fn create_cw20_transfer_msg(
    cw20_address: &Addr,
    recipient: &Addr,
    amount: Uint128,
) -> Option<WasmMsg> {
    let msg = to_json_binary(&Cw20ExecuteMsg::Transfer {
        recipient: recipient.to_string(),
        amount,
    })
    .ok()?;

    Some(WasmMsg::Execute {
        contract_addr: cw20_address.to_string(),
        msg,
        funds: vec![],
    })
}

pub fn create_bank_transfer_msg(
    recipient: &Addr,
    token_denom: &TokenDenom,
    amount: Uint128,
) -> BankMsg {
    BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom: token_denom.to_string(),
            amount,
        }],
    }
}
