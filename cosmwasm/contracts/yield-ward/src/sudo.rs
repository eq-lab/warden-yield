use crate::state::{IBC_SEND_MESSAGE_INFLIGHT, IBC_SEND_MESSAGE_RECOVERY};
use crate::types::IbcSendMessageStatus;
use crate::ContractError;
use cosmwasm_std::{DepsMut, Event, Response};

pub fn execute_receive_lifecycle_completion(
    _deps: DepsMut,
    action: &str,
    fail_status: IbcSendMessageStatus,
    source_channel_id: &String,
    sequence: u64,
    success: bool,
) -> Result<Response, ContractError> {
    let status = match fail_status {
        IbcSendMessageStatus::Sent => "Sent",
        IbcSendMessageStatus::AckSuccess => "AckSuccess",
        IbcSendMessageStatus::AckFailure => "AckFailure",
        IbcSendMessageStatus::TimedOut => "TimedOut",
    };
    let response = Response::new().add_attribute("action", action).add_event(
        Event::new("lifecycle_response")
            .add_attribute("fail_status", status)
            .add_attribute("source_channel_id", source_channel_id)
            .add_attribute("sequence", sequence.to_string())
            .add_attribute("success", success.to_string()),
    );

    return Ok(response);
    // todo: update storage, return funds
    // let sent_message =
    //     IBC_SEND_MESSAGE_INFLIGHT.may_load(deps.storage, (source_channel_id, sequence))?;
    // let Some(inflight_sent_message) = sent_message else {
    //     return Ok(response.add_attribute("status", "unexpected call"));
    // };
    //
    // IBC_SEND_MESSAGE_INFLIGHT.remove(deps.storage, (source_channel_id, sequence));
    //
    // if success {
    //     return Ok(response.add_attribute("status", "message successfully delivered"));
    // }
    //
    // let mut recovery = inflight_sent_message;
    // let sender = recovery.sender.clone();
    //
    // IBC_SEND_MESSAGE_RECOVERY.update(deps.storage, &sender, |items| {
    //     recovery.status = fail_status;
    //     let Some(mut items) = items else {
    //         return Ok::<_, ContractError>(vec![recovery]);
    //     };
    //     items.push(recovery);
    //     Ok(items)
    // })?;
    //
    // Ok(response.add_attribute(
    //     "status",
    //     format!("recovery created for address: {:?}", sender),
    // ))
}
