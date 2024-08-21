use crate::state::{ContractConfigState, QueueParams, StakeItem, StakeStatsItem, UnstakeItem};
use crate::types::{TokenConfig, TokenDenom};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint128};
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub tokens: Vec<(TokenDenom, TokenConfig)>,
    pub axelar: Addr,
    pub lp_token_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Stake,
    Unstake,
    Reinit {
        token_denom: TokenDenom,
    },
    MintLpToken {
        recipient: Addr,
        lp_token_address: Addr,
        amount: Uint128,
    },
    // todo: add method to disable MintLpToken method
    AddToken {
        token_denom: TokenDenom,
        is_stake_enabled: bool,
        is_unstake_enabled: bool,
        chain: String,
        symbol: String,
        name: String,
        evm_yield_contract: String,
        evm_address: String,
        lp_token_denom: String, // todo: remove it, redundant
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

    DisallowMint,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetContractConfigResponse)]
    ContractConfig,
    #[returns(GetTokensConfigsResponse)]
    TokensConfigs,
    #[returns(GetStakeStatsResponse)]
    StakeStats,
    #[returns(GetQueueParamsResponse)]
    StakeParams { token_denom: TokenDenom },
    #[returns(GetQueueParamsResponse)]
    UnstakeParams { token_denom: TokenDenom },
    #[returns(GetStakeItemResponse)]
    StakeElem { token_denom: TokenDenom, id: u64 },
    #[returns(GetUnstakeItemResponse)]
    UnstakeElem { token_denom: TokenDenom, id: u64 },
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
pub struct GetStakeStatsResponse {
    pub stats: Vec<(TokenDenom, StakeStatsItem)>,
}

#[cw_serde]
pub struct GetStakeItemResponse {
    pub item: StakeItem,
}

#[cw_serde]
pub struct GetUnstakeItemResponse {
    pub item: UnstakeItem,
}

#[cw_serde]
pub struct GetQueueParamsResponse {
    pub params: QueueParams,
}

#[cw_serde]
pub enum MigrateMsg {}

#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct MsgLpTokenMintResponse {}
