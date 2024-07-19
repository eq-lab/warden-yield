use crate::state::{
    ContractConfigState, QueueParams, StakeQueueItem, TokenStats, UnstakeQueueItem,
};
use crate::types::{TokenConfig, TokenDenom};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub tokens: Vec<(TokenDenom, TokenConfig)>,
    pub axelar: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Stake,
    Unstake,
    Reinit {
        token_denom: TokenDenom,
    },

    AddToken {
        token_denom: TokenDenom,
        config: TokenConfig,
    },
    UpdateTokenConfig {
        token_denom: TokenDenom,
        config: TokenConfig,
    },

    HandleResponse {
        source_chain: String,
        source_address: String,
        payload: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetContractConfigResponse)]
    ContractConfig,
    #[returns(GetTokensConfigsResponse)]
    TokensConfigs,
    #[returns(GetTokensStatsResponse)]
    TokensStats,
    #[returns(GetQueueParamsResponse)]
    StakeQueueParams { token_denom: TokenDenom },
    #[returns(GetQueueParamsResponse)]
    UnstakeQueueParams { token_denom: TokenDenom },
    #[returns(GetStakeQueueItemResponse)]
    StakeQueueElem { token_denom: TokenDenom, id: u64 },
    #[returns(GetUnstakeQueueItemResponse)]
    UnstakeQueueElem { token_denom: TokenDenom, id: u64 },
}

#[cw_serde]
pub struct GetContractConfigResponse {
    pub config: ContractConfigState,
}

#[cw_serde]
pub struct GetTokensConfigsResponse {
    pub tokens: Vec<(TokenDenom, TokenConfig)>,
}

#[cw_serde]
pub struct GetTokensStatsResponse {
    pub stats: Vec<(TokenDenom, TokenStats)>,
}

#[cw_serde]
pub struct GetStakeQueueItemResponse {
    pub item: StakeQueueItem,
}

#[cw_serde]
pub struct GetUnstakeQueueItemResponse {
    pub item: UnstakeQueueItem,
}

#[cw_serde]
pub struct GetQueueParamsResponse {
    pub params: QueueParams,
}

#[cw_serde]
pub enum MigrateMsg {}

#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct MsgLpTokenMintResponse {}
