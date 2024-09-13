use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::ExecuteMsg;
use crate::state::{
    CONTRACT_CONFIG, TOKEN_CONFIG, TOKEN_DENOM_BY_LPT_ADDRESS, TOKEN_DENOM_BY_SOURCE,
};
use crate::types::{TokenConfig, TokenDenom};
use crate::ContractError;
use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, Deps, MessageInfo, StdResult, WasmMsg};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

pub fn assert_msg_sender_is_admin(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let contract_config = CONTRACT_CONFIG.load(deps.storage)?;
    if contract_config.owner != info.sender {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

pub fn assert_msg_sender_is_axelar(deps: Deps, sender: &Addr) -> Result<(), ContractError> {
    // todo: check origin assertion: https://github.com/axelarnetwork/evm-cosmos-gmp-sample/tree/main/cosmwasm-integration#authenticate-the-sender
    let contract_config = CONTRACT_CONFIG.load(deps.storage)?;
    if contract_config.axelar != sender {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

pub fn find_token_by_message_source(
    deps: Deps,
    source_chain: &String,
    source_address: &String,
) -> Result<(TokenDenom, TokenConfig), ContractError> {
    let token_denom = TOKEN_DENOM_BY_SOURCE
        .may_load(deps.storage, (&source_chain, &source_address))?
        .ok_or(ContractError::UnknownTokenBySource {
            source_chain: source_chain.clone(),
            source_address: source_address.clone(),
        })?;

    let token_config = TOKEN_CONFIG.load(deps.storage, &token_denom)?;

    return Ok((token_denom, token_config));
}

pub fn find_deposit_token_denom_by_lpt_address(
    deps: Deps,
    lpt_address: &Addr,
) -> Result<TokenDenom, ContractError> {
    Ok(TOKEN_DENOM_BY_LPT_ADDRESS
        .load(deps.storage, lpt_address)
        .map_err(|_| ContractError::UnknownLpToken(lpt_address.to_string()))?)
}
