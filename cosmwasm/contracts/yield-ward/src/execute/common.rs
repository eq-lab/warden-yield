use crate::state::TOKEN_CONFIG;
use crate::types::TokenDenom;
use crate::ContractError;
use cosmwasm_std::{to_json_binary, Addr, Deps, Uint128, WasmMsg};
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
    .unwrap();

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
    .unwrap();

    Some(WasmMsg::Execute {
        contract_addr: cw20_address.to_string(),
        msg,
        funds: vec![],
    })
}

pub fn assert_cw20_deposit_token_address(
    deps: Deps,
    token_denom: &TokenDenom,
    actual_cw20_address: &Addr,
) -> Result<(), ContractError> {
    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &token_denom)?
        .ok_or(ContractError::UnknownToken(token_denom.clone()))?;
    if token_config.cw20_address != actual_cw20_address {
        return Err(ContractError::MismatchCw20Token {
            actual: actual_cw20_address.to_string(),
            expected: token_config.cw20_address.to_string(),
        });
    }
    Ok(())
}

pub fn assert_cw20_lpt_address(
    deps: Deps,
    token_denom: &TokenDenom,
    actual_cw20_address: &Addr,
) -> Result<(), ContractError> {
    let token_config = TOKEN_CONFIG
        .may_load(deps.storage, &token_denom)?
        .ok_or(ContractError::UnknownToken(token_denom.clone()))?;
    if token_config.lpt_address != actual_cw20_address {
        return Err(ContractError::MismatchCw20Token {
            actual: actual_cw20_address.to_string(),
            expected: token_config.cw20_address.to_string(),
        });
    }
    Ok(())
}
