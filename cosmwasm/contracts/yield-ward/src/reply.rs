use crate::state::{IBC_SEND_MESSAGE_INFLIGHT, IBC_SEND_MESSAGE_REPLY};
use crate::types::{IbcMsgTransferResponse, IbcSendMessageStatus, IbcSendMessageTransfer};
use crate::ContractError;
use cosmwasm_std::{DepsMut, Event, Reply, Response, SubMsgResult};
use prost::Message;

pub fn execute_send_ibc_message_reply(
    deps: DepsMut,
    message: Reply,
) -> Result<Response, ContractError> {
    // match message.result {
    //     SubMsgResult::Ok(response)
    //         if response.msg_responses.is_empty() && response.msg_responses.is_empty() =>
    //     {
    //         // Ok(Binary::new(vec![]))
    //         return Ok(Response::new()
    //             .add_attribute("response_empty", "true")
    //             .add_attribute("message.gas_used", message.gas_used.to_string()));
    //     }
    //     SubMsgResult::Ok(response) if response.msg_responses.is_empty() => {
    //         // #[allow(deprecated)]
    //         // Ok(response.data.unwrap())
    //         return Ok(Response::new()
    //             .add_attribute("response.msg_responses.empty", "true")
    //             .add_attribute(
    //                 "response.data.len",
    //                 response.data.unwrap().len().to_string(),
    //             )
    //             .add_attribute("message.gas_used", message.gas_used.to_string()));
    //     }
    //     SubMsgResult::Ok(response) if !response.msg_responses.is_empty() => {
    //         // Ok(response.msg_responses[0].value.clone())
    //         return Ok(Response::new()
    //             .add_attribute("response.msg_responses.empty", "false")
    //             .add_attribute("message.gas_used", message.gas_used.to_string()));
    //     }
    //     SubMsgResult::Ok(_) | SubMsgResult::Err(_) => Err(ContractError::IbcMessageFailed {
    //         message: format!("failed reply: {:?}", message.result),
    //     }),
    // }

    let reply_result = &match message.result {
        SubMsgResult::Ok(response) if !response.msg_responses.is_empty() => {
            Ok(response.msg_responses[0].value.clone())
        }
        SubMsgResult::Ok(_) | SubMsgResult::Err(_) => Err(ContractError::IbcMessageFailed {
            message: format!("failed reply: {:?}", message.result),
        }),
    }?;

    let response = IbcMsgTransferResponse::decode(&reply_result[..]).map_err(|_e| {
        ContractError::IbcMessageFailed {
            message: format!("failed to decode reply result: {reply_result}"),
        }
    })?;

    let reply = IBC_SEND_MESSAGE_REPLY.load(deps.storage)?;
    IBC_SEND_MESSAGE_REPLY.remove(deps.storage);

    IBC_SEND_MESSAGE_INFLIGHT.save(
        deps.storage,
        (&reply.channel_id, response.sequence),
        &IbcSendMessageTransfer {
            channel_id: reply.channel_id.clone(),
            sequence: response.sequence,
            denom: reply.denom.clone(),
            action_type: reply.action_type.clone(),
            action_id: reply.action_id,
            status: IbcSendMessageStatus::Sent,
        },
    )?;

    Ok(Response::new().add_event(
        Event::new("ibc_reply")
            .add_attribute("channel_id", reply.channel_id)
            .add_attribute("sequence", response.sequence.to_string())
            .add_attribute("action_type", reply.action_type.to_string())
            .add_attribute("action_id", reply.action_id.to_string())
            .add_attribute("denom", reply.denom.unwrap()),
    ))
}
