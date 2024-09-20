use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use enum_repr::EnumRepr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type TokenDenom = String;

#[cw_serde]
pub struct TokenConfig {
    pub is_stake_enabled: bool,
    pub is_unstake_enabled: bool,
    pub deposit_token_symbol: String,
    pub chain: String,
    pub evm_yield_contract: String,
    pub evm_address: String,
    pub lpt_symbol: String,
    pub lpt_address: Addr,
}

#[EnumRepr(type = "u64")]
pub enum ReplyType {
    SendIbcMessage = 1,
}

impl TryFrom<&u64> for ReplyType {
    type Error = ();

    fn try_from(value: &u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ReplyType::SendIbcMessage),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StakeResponseData {
    pub status: Status,
    pub stake_id: u64,
    pub reinit_unstake_id: u64,
    pub lp_token_amount: Uint128,
}

#[derive(Debug, PartialEq)]
pub struct UnstakeResponseData {
    pub status: Status,
    pub unstake_id: u64,
    pub reinit_unstake_id: u64,
}

#[derive(Debug, PartialEq)]
pub struct ReinitResponseData {
    pub reinit_unstake_id: u64,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Status {
    Success = 0,
    Fail,
}

impl TryFrom<&u8> for Status {
    type Error = ();

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Status::Success),
            1 => Ok(Status::Fail),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ActionType {
    Stake = 0,
    Unstake,
    Reinit,
}

impl ActionType {
    pub fn to_string(&self) -> String {
        match self {
            ActionType::Stake => "stake".to_string(),
            ActionType::Unstake => "unstake".to_string(),
            ActionType::Reinit => "reinit".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum StakeActionStage {
    WaitingExecution = 0,
    Executed,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum UnstakeActionStage {
    WaitingRegistration = 0,
    Registered,
    Executed,
    Failed,
}

#[cw_serde]
pub enum IbcSendMessageStatus {
    Sent,
    AckSuccess,
    AckFailure,
    TimedOut,
}

#[cw_serde]
pub struct IbcSendMessageTransfer {
    pub channel_id: String,
    pub sequence: u64,
    pub denom: Option<String>,
    pub action_type: ActionType,
    pub action_id: u64,
    pub status: IbcSendMessageStatus,
}

#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct IbcMsgTransferResponse {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}

#[cw_serde]
pub struct IbcSendMessageReply {
    pub channel_id: String,
    pub recipient: String,
    pub denom: Option<String>,
    pub action_type: ActionType,
    pub action_id: u64,
    pub block_time: Timestamp,
}
