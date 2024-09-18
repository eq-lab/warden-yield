use cosmwasm_std::{Binary, Coin, Deps, Env, IbcMsg, MessageInfo, Response};
use serde_json_wasm::to_string;

use crate::{
    msg::{GmpMessage, GmpMsgType},
    state::AXELAR_CONFIG,
    ContractError,
};

pub fn send_message_evm(
    deps: Deps,
    env: Env,
    info: &MessageInfo,
    payload: Binary,
) -> Result<Response, ContractError> {
    let axelar_config = AXELAR_CONFIG
        .load(deps.storage)
        .map_err(|_| ContractError::CustomError("Failed to load axelar config".into()))?;

    // info.funds.len() == 0 -- revert
    // info.funds.len == 2 -- stake call
    // info.funds.len == 1 -- unstake or reinit call
    // info.funds.len() > 2 -- revert
    // or some other limits: depends on the axelar fee token
    // TODO axelar fee: seems like some checks, similar to the above ones, are required
    let fund = info.funds.first();

    // TODO axelar fee: in ward tokens)? subtract from coin amount?
    // Feels like the `type_` value can be actually defined by info.funds length
    let (transfer, fee, type_) = match fund {
        Some(coin) => (coin, None, GmpMsgType::WithToken as i64),
        None => (&Coin::new(0_u64, "ward"), None, GmpMsgType::Pure as i64),
    };

    let gmp_message: GmpMessage = GmpMessage {
        destination_chain: axelar_config.evm_destination_chain_tag,
        destination_address: axelar_config.yield_ward_evm_address,
        payload,
        type_,
        fee,
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
        amount: transfer.clone(),
        timeout: env
            .block
            .time
            .plus_seconds(axelar_config.ibc_timeout_seconds)
            .into(),
        memo,
    };

    Ok(Response::new().add_message(ibc_message))
}
