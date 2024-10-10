use cosmwasm_std::{Binary, Coin, Deps, Env, IbcMsg, Response, Uint128};
use serde_json_wasm::to_string;

use crate::{
    msg::{Fee, GmpMessage, GmpMsgType},
    state::AXELAR_CONFIG,
    types::TokenConfig,
    ContractError,
};

pub fn send_message_evm(
    deps: Deps,
    env: Env,
    fund: &Coin,
    token_config: &TokenConfig,
    payload: Binary,
    fee_amount: Uint128,
) -> Result<Response, ContractError> {
    let axelar_config = AXELAR_CONFIG
        .load(deps.storage)
        .map_err(|_| ContractError::CustomError("Failed to load axelar config".into()))?;

    let type_ = match fund.amount == fee_amount {
        true => GmpMsgType::Pure as i64,
        false => GmpMsgType::WithToken as i64,
    };

    let fee = Some(Fee {
        amount: fee_amount.to_string(),
        recipient: axelar_config.axelar_fee_recipient_address,
    });

    let gmp_message: GmpMessage = GmpMessage {
        destination_chain: token_config.chain.clone(),
        destination_address: token_config.evm_yield_contract.clone(),
        payload,
        type_,
        fee,
        ibc_callback: env.contract.address.to_string(),
    };

    let memo = to_string(&gmp_message).ok();
    if memo.is_none() {
        return Err(ContractError::CustomError(
            "Failed to serialize gmp message".into(),
        ));
    }

    let ibc_message = IbcMsg::Transfer {
        channel_id: axelar_config.axelar_channel_id,
        to_address: axelar_config.axelar_gateway_cosmos_address,
        amount: fund.clone(),
        timeout: env
            .block
            .time
            .plus_seconds(axelar_config.ibc_timeout_seconds)
            .into(),
        memo,
    };

    Ok(Response::new().add_message(ibc_message))
}
