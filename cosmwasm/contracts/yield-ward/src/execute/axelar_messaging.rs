use cosmwasm_std::{
    Binary, Coin, CosmosMsg, DepsMut, Env, IbcMsg, MessageInfo, Response, SubMsg,
    TransferMsgBuilder, Uint128,
};
use serde_json_wasm::to_string;

use crate::msg::Fee;
use crate::state::IBC_SEND_MESSAGE_REPLY;
use crate::types::{ActionType, IbcSendMessageReply, ReplyType};
use crate::{
    msg::{GmpMessage, GmpMsgType},
    state::AXELAR_CONFIG,
    types::TokenConfig,
    ContractError,
};

pub fn send_message_evm(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    token_config: &TokenConfig,
    payload: Binary,
    action_id: u64,
    action_type: ActionType,
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
        Some(coin) => (coin, coin, GmpMsgType::WithToken as i64),
        None => (
            &Coin::new(0_u64, "ward"),
            &Coin::new(0_u64, "ward"),
            GmpMsgType::Pure as i64,
        ),
    };

    // let fee_amount = Uint128::from(150000_u128);
    let gmp_message: GmpMessage = GmpMessage {
        destination_chain: token_config.chain.clone(),
        destination_address: token_config.evm_yield_contract.clone(),
        payload,
        type_,
        fee: None, // Some(Fee {
                   //     amount: fee_amount.to_string(),
                   //     recipient: "axelar1zl3rxpp70lmte2xr6c4lgske2fyuj3hupcsvcd".to_string(),
                   // }),
    };

    let memo = to_string(&gmp_message).ok();
    if memo.is_none() {
        return Err(ContractError::CustomError(
            "Failed to serialize gmp message".into(),
        ));
    }

    let ibc_message = IbcMsg::Transfer {
        channel_id: axelar_config.axelar_channel_id.clone(),
        to_address: axelar_config.axelar_gateway_cosmos_address.clone(),
        amount: transfer.clone(),
        timeout: env
            .block
            .time
            .plus_seconds(axelar_config.ibc_timeout_seconds)
            .into(),
        memo,
    };

    let ibc_sub_message = SubMsg::reply_on_success(ibc_message, ReplyType::SendIbcMessage.repr());
    let reply = IBC_SEND_MESSAGE_REPLY.may_load(deps.storage)?;
    if reply.is_some() {
        return Err(ContractError::IbcSubMessageFailed {
            message: "reply is already exist".to_string(),
        });
    }

    IBC_SEND_MESSAGE_REPLY.save(
        deps.storage,
        &IbcSendMessageReply {
            channel_id: axelar_config.axelar_channel_id,
            recipient: axelar_config.axelar_gateway_cosmos_address,
            denom: Some(transfer.denom.clone()), // todo: support Reinit with no tokens to send
            action_id,
            action_type,
            block_time: env.block.time,
        },
    )?;
    Ok(Response::new().add_submessage(ibc_sub_message))
}
