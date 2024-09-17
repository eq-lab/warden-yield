use cosmwasm_std::{Binary, Deps, Env, IbcMsg, MessageInfo, Response};
use serde_json_wasm::to_string;

use crate::{msg::GmpMessage, state::AXELAR_CONFIG, ContractError};

pub fn send_message_evm(
    deps: Deps,
    env: Env,
    info: &MessageInfo,
    payload: Binary,
) -> Result<Response, ContractError> {
    // {info.funds} used to pay gas. Must only contain 1 token type.
    // let coin: Coin = cw_utils::one_coin(&info).unwrap();

    let axelar_config = AXELAR_CONFIG
        .load(deps.storage)
        .map_err(|_| ContractError::CustomError("Failed to load axelar config".into()))?;

    let gmp_message: GmpMessage = GmpMessage {
        destination_chain: axelar_config.evm_destination_chain_tag,
        destination_address: axelar_config.yield_ward_evm_address,
        payload: payload,
        type_: 1,
        fee: None,
    };

    let memo = Some(
        to_string(&gmp_message)
            .map_err(|_| ContractError::CustomError("Failed to serialize gmp message".into()))?,
    );

    let ibc_message = IbcMsg::Transfer {
        channel_id: axelar_config.axelar_channel_id,
        to_address: axelar_config.axelar_gateway_cosmos_address,
        amount: info.funds.first().unwrap().clone(),
        timeout: env
            .block
            .time
            .plus_seconds(axelar_config.ibc_timeout_seconds)
            .into(),
        memo: memo,
    };

    Ok(Response::new().add_message(ibc_message))
}
