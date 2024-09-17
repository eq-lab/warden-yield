use cosmwasm_std::{Binary, Deps, Env, IbcMsg, MessageInfo, Response};
use serde_json_wasm::to_string;

use crate::{msg::GmpMessage, ContractError};

pub fn send_message_evm(
    _deps: Deps,
    env: Env,
    info: &MessageInfo,
    payload: Binary,
) -> Result<Response, ContractError> {
    // {info.funds} used to pay gas. Must only contain 1 token type.
    // let coin: Coin = cw_utils::one_coin(&info).unwrap();

    let gmp_message: GmpMessage = GmpMessage {
        destination_chain: "evm".into(),
        destination_address: "evmAddress".into(),
        payload: payload,
        type_: 1,
        fee: None,
    };

    let memo = Some(
        to_string(&gmp_message)
            .map_err(|_| ContractError::CustomError("Failed to serialize gmp message".into()))?,
    );

    let ibc_message = IbcMsg::Transfer {
        channel_id: "channel-3".to_string(),
        to_address: "axelar1dv4u5k73pzqrxlzujxg3qp8kvc3pje7jtdvu72npnt5zhq05ejcsn5qme5".to_string(),
        amount: info.funds.first().unwrap().clone(),
        timeout: env.block.time.plus_seconds(604_800u64).into(), // week, taken from example
        memo: memo,
    };

    Ok(Response::new().add_message(ibc_message))
}
