use crate::state::{ContractConfigState, TokenStats};
use crate::types::{StakeStatus, TokenConfig, TokenDenom};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub tokens: Vec<(TokenDenom, TokenConfig)>,
    pub axelar: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Stake,
    Unstake {
        token_denom: TokenDenom,
    },

    AddToken {
        token_denom: String,
        config: TokenConfig,
    },
    UpdateTokenConfig {
        token_denom: String,
        config: TokenConfig,
    },

    HandleStakeResponse {
        account: Addr,
        token_evm: String,
        token_amount: Uint128,
        shares_amount: Uint128,
        status: StakeStatus,
    },
    HandleUnstakeResponse {
        account: Addr,
        token_evm: String,
        token_amount: Uint128,
        shares_amount: Uint128,
        status: StakeStatus,
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
    #[returns(GetUserStatsResponse)]
    UserStats {
        account: Addr,
        token_denom: TokenDenom,
    },
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
pub struct GetUserStatsResponse {
    pub stats: TokenStats,
}

#[cw_serde]
pub enum MigrateMsg {}

#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct MsgLpTokenMintResponse {}
