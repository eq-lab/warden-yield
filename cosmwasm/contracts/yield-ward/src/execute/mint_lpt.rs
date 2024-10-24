use crate::execute::common::create_cw20_mint_msg;
use crate::helpers::assert_msg_sender_is_admin;
use crate::state::CONTRACT_CONFIG;
use crate::ContractError;
use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Response, Uint128};

pub fn try_mint_lp_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Addr,
    lp_token_address: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    if !CONTRACT_CONFIG.load(deps.storage)?.is_mint_allowed {
        return Err(ContractError::MintIsNotAllowed);
    }

    let mint_msg = create_cw20_mint_msg(&lp_token_address, &recipient, amount)?;

    Ok(Response::new()
        .add_event(
            Event::new("mint_lpt")
                .add_attribute("lpt_address", lp_token_address)
                .add_attribute("to", recipient)
                .add_attribute("amount", amount.to_string()),
        )
        .add_message(mint_msg))
}

pub fn try_disallow_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    let mut contract_config = CONTRACT_CONFIG.load(deps.storage)?;
    contract_config.is_mint_allowed = false;
    CONTRACT_CONFIG.save(deps.storage, &contract_config)?;

    Ok(Response::new().add_event(Event::new("disallow_mint")))
}
