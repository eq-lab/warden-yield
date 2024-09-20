use cosmwasm_std::{
    from_json, to_json_binary, Binary, DepsMut, Env, Event, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse,
    IbcDestinationCallbackMsg, IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, IbcSourceCallbackMsg, StdAck, StdResult,
};
//
// use crate::state::IBC_CONNECTION_COUNTS;
// use crate::{state::IBC_TIMEOUT_COUNTS, ContractError};
//
// pub const IBC_VERSION: &str = "counter-1";
//
// /// Handles the `OpenInit` and `OpenTry` parts of the IBC handshake.
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn ibc_channel_open(
//     _deps: DepsMut,
//     _env: Env,
//     msg: IbcChannelOpenMsg,
// ) -> Result<IbcChannelOpenResponse, ContractError> {
//     validate_order_and_version(msg.channel(), msg.counterparty_version())?;
//     Ok(None)
// }
//
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn ibc_channel_connect(
//     deps: DepsMut,
//     _env: Env,
//     msg: IbcChannelConnectMsg,
// ) -> Result<IbcBasicResponse, ContractError> {
//     validate_order_and_version(msg.channel(), msg.counterparty_version())?;
//
//     // Initialize the count for this channel to zero.
//     let channel = msg.channel().endpoint.channel_id.clone();
//     IBC_CONNECTION_COUNTS.save(deps.storage, channel.clone(), &0)?;
//
//     Ok(IbcBasicResponse::new()
//         .add_attribute("method", "ibc_channel_connect")
//         .add_attribute("channel_id", channel))
// }
//
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn ibc_channel_close(
//     deps: DepsMut,
//     _env: Env,
//     msg: IbcChannelCloseMsg,
// ) -> Result<IbcBasicResponse, ContractError> {
//     let channel = msg.channel().endpoint.channel_id.clone();
//     // Reset the state for the channel.
//     IBC_CONNECTION_COUNTS.remove(deps.storage, channel.clone());
//     Ok(IbcBasicResponse::new()
//         .add_attribute("method", "ibc_channel_close")
//         .add_attribute("channel", channel))
// }
//
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn ibc_packet_receive(
//     deps: DepsMut,
//     env: Env,
//     msg: IbcPacketReceiveMsg,
// ) -> Result<IbcReceiveResponse, ContractError> {
//     // Regardless of if our processing of this packet works we need to
//     // commit an ACK to the chain. As such, we wrap all handling logic
//     // in a seprate function and on error write out an error ack.
//     match do_ibc_packet_receive(deps, env, msg) {
//         Ok(response) => Ok(response),
//         Err(error) => Ok(IbcReceiveResponse::new(make_ack_fail(error.to_string()))
//             .add_attribute("method", "ibc_packet_receive")
//             .add_attribute("error", error.to_string())),
//     }
// }
//
// pub fn do_ibc_packet_receive(
//     _deps: DepsMut,
//     _env: Env,
//     msg: IbcPacketReceiveMsg,
// ) -> Result<IbcReceiveResponse, ContractError> {
//     // The channel this packet is being relayed along on this chain.
//     let _channel = msg.packet.dest.channel_id;
//     // todo
//     // let msg: IbcExecuteMsg = from_json(&msg.packet.data)?;
//     //
//     // match msg {
//     //     IbcExecuteMsg::Increment {} => execute_increment(deps, channel),
//     // }
//     Ok(IbcReceiveResponse::new(make_ack_success()).add_event(Event::new("ibc_packet_receive")))
// }
//
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn ibc_packet_ack(
//     _deps: DepsMut,
//     _env: Env,
//     msg: IbcPacketAckMsg,
// ) -> Result<IbcBasicResponse, ContractError> {
//     // this example assumes that the acknowledgement is an StdAck
//     let ack: StdResult<StdAck> = from_json(&msg.acknowledgement.data);
//     if ack.is_err() {
//         return Ok(IbcBasicResponse::new().add_event(
//             Event::new("ibc_packet_ack")
//                 .add_attribute("success", "no_info")
//                 .add_attribute("error", "from_json error!"),
//         ));
//     }
//     let (is_ok, err) = match ack.unwrap() {
//         StdAck::Success(_) => (true, "".to_string()),
//         StdAck::Error(x) => (false, x),
//     };
//     // here you can do something with the acknowledgement
//
//     Ok(IbcBasicResponse::new().add_event(
//         Event::new("ibc_packet_ack")
//             .add_attribute("success", is_ok.to_string())
//             .add_attribute("error", err),
//     ))
// }
//
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn ibc_packet_timeout(
//     deps: DepsMut,
//     _env: Env,
//     msg: IbcPacketTimeoutMsg,
// ) -> Result<IbcBasicResponse, ContractError> {
//     IBC_TIMEOUT_COUNTS.update(
//         deps.storage,
//         // timed out packets are sent by us, so lookup based on packet
//         // source, not destination.
//         msg.packet.src.channel_id.clone(),
//         |count| -> StdResult<_> { Ok(count.unwrap_or_default() + 1) },
//     )?;
//     // As with ack above, nothing to do here. If we cared about
//     // keeping track of state between the two chains then we'd want to
//     // respond to this likely as it means that the packet in question
//     // isn't going anywhere.
//     // Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_timeout"))
//
//     let response = IbcBasicResponse::new()
//         .add_attribute("action", "ibc_packet_timeout")
//         .add_event(
//             Event::new("lifecycle_response")
//                 .add_attribute("fail_status", "timed_out")
//                 .add_attribute("source_channel_id", msg.packet.src.channel_id)
//                 .add_attribute("sequence", msg.packet.sequence.to_string()),
//         );
//
//     return Ok(response);
// }
//
// pub fn validate_order_and_version(
//     channel: &IbcChannel,
//     counterparty_version: Option<&str>,
// ) -> Result<(), ContractError> {
//     // We expect an unordered channel here. Ordered channels have the
//     // property that if a message is lost the entire channel will stop
//     // working until you start it again.
//     if channel.order != IbcOrder::Unordered {
//         return Err(ContractError::IbcOrderedChannel);
//     }
//
//     if channel.version != IBC_VERSION {
//         return Err(ContractError::InvalidVersion {
//             actual: channel.version.to_string(),
//             expected: IBC_VERSION.to_string(),
//         });
//     }
//
//     // Make sure that we're talking with a counterparty who speaks the
//     // same "protocol" as us.
//     //
//     // For a connection between chain A and chain B being established
//     // by chain A, chain B knows counterparty information during
//     // `OpenTry` and chain A knows counterparty information during
//     // `OpenAck`. We verify it when we have it but when we don't it's
//     // alright.
//     if let Some(counterparty_version) = counterparty_version {
//         if counterparty_version != IBC_VERSION {
//             return Err(ContractError::InvalidVersion {
//                 actual: counterparty_version.to_string(),
//                 expected: IBC_VERSION.to_string(),
//             });
//         }
//     }
//
//     Ok(())
// }
//
// /// IBC ACK. See:
// /// https://github.com/cosmos/cosmos-sdk/blob/f999b1ff05a4db4a338a855713864497bedd4396/proto/ibc/core/channel/v1/channel.proto#L141-L147
// #[cw_serde]
// pub enum Ack {
//     Result(Binary),
//     Error(String),
// }
//
// pub fn make_ack_success() -> Binary {
//     let res = Ack::Result(b"1".into());
//     to_json_binary(&res).unwrap()
// }
//
// pub fn make_ack_fail(err: String) -> Binary {
//     let res = Ack::Error(err);
//     to_json_binary(&res).unwrap()
// }

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

/// This is the entrypoint that is called by the source chain when a callbacks-enabled IBC message
/// is acknowledged or times out.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_source_callback(
    deps: DepsMut,
    _env: Env,
    msg: IbcSourceCallbackMsg,
) -> StdResult<IbcBasicResponse> {
    let response = IbcBasicResponse::new().add_attribute("action", "ibc_source_callback");

    match msg {
        IbcSourceCallbackMsg::Acknowledgement(ack) => {
            // save the ack
            Ok(response.add_event(Event::new("ibc_ack")))
        }
        IbcSourceCallbackMsg::Timeout(timeout) => {
            // save the timeout
            Ok(response.add_event(
                Event::new("ibc_timeout")
                    .add_attribute("seqno", timeout.packet.sequence.to_string()),
            ))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_destination_callback(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcDestinationCallbackMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_destination_callback"))
}
