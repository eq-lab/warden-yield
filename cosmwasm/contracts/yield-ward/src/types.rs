use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
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

pub enum ReplyType {
    LpMint = 1,
}

impl TryFrom<&u64> for ReplyType {
    type Error = ();

    fn try_from(value: &u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ReplyType::LpMint),
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

pub enum ActionType {
    Stake = 0,
    Unstake,
    Reinit,
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
