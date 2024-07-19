use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::ExecuteMsg;
use crate::state::{CONTRACT_CONFIG_STATE, TOKENS_CONFIGS_STATE};
use crate::types::{TokenConfig, TokenDenom};
use crate::ContractError;
use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, Deps, MessageInfo, Order, StdResult, WasmMsg};

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
    let contract_config = CONTRACT_CONFIG_STATE.load(deps.storage)?;
    if contract_config.owner != info.sender {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

pub fn assert_msg_sender_is_axelar(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let contract_config = CONTRACT_CONFIG_STATE.load(deps.storage)?;
    if contract_config.axelar != info.sender {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

pub fn find_token_by_message_source(
    deps: Deps,
    source_chain: &String,
    source_address: &String,
) -> Result<(TokenDenom, TokenConfig), ContractError> {
    let tokens_configs: StdResult<Vec<(TokenDenom, TokenConfig)>> = TOKENS_CONFIGS_STATE
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let tokens_configs = tokens_configs?;

    tokens_configs
        .iter()
        .find(|(_, config)| {
            config.chain.to_lowercase() == source_chain.to_lowercase()
                && config.evm_yield_contract.to_lowercase() == source_address.to_lowercase()
        })
        .cloned()
        .ok_or(ContractError::UnknownTokenBySource {
            source_chain: source_chain.clone(),
            source_address: source_address.clone(),
        })
}

pub fn find_token_by_lp_token_denom(
    deps: Deps,
    lp_token_denom: &String,
) -> Result<(TokenDenom, TokenConfig), ContractError> {
    let tokens_configs: StdResult<Vec<(TokenDenom, TokenConfig)>> = TOKENS_CONFIGS_STATE
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let tokens_configs = tokens_configs?;

    tokens_configs
        .iter()
        .find(|(_, config)| &config.lp_token_denom == lp_token_denom)
        .cloned()
        .ok_or(ContractError::UnknownLpToken(lp_token_denom.clone()))
}
