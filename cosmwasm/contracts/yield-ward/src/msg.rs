use crate::state::{ContractConfigState, QueueParams, StakeItem, StakeStatsItem, UnstakeItem};
use crate::types::{TokenConfig, TokenDenom};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint128};
use cw20::Cw20ReceiveMsg;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub axelar: Addr,
    pub lp_token_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Stake,
    Receive(Cw20ReceiveMsg),
    Reinit {
        token_denom: TokenDenom,
    },
    MintLpToken {
        recipient: Addr,
        lp_token_address: Addr,
        amount: Uint128,
    },
    AddToken {
        token_denom: TokenDenom,
        token_symbol: String,
        token_decimals: u8,
        is_stake_enabled: bool,
        is_unstake_enabled: bool,
        chain: String,
        evm_yield_contract: String,
        evm_address: String,
        lpt_symbol: String,
        lpt_name: String,
    },
    UpdateContractConfig {
        contract_config: ContractConfigState,
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
    ContractConfig {},
    #[returns(GetTokensConfigsResponse)]
    TokensConfigs {},
    #[returns(GetStakeStatsResponse)]
    StakeStats {},
    #[returns(GetQueueParamsResponse)]
    StakeParams { token_denom: TokenDenom },
    #[returns(GetQueueParamsResponse)]
    UnstakeParams { token_denom: TokenDenom },
    #[returns(GetStakeItemResponse)]
    StakeElem { token_denom: TokenDenom, id: u64 },
    #[returns(GetUnstakeItemResponse)]
    UnstakeElem { token_denom: TokenDenom, id: u64 },
    #[returns(GetTokenDenomBySourceResponse)]
    TokenDenomBySource {},
    #[returns(GetTokenDenomByLptAddressResponse)]
    TokenDenomByLptAddress {},
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
pub struct GetTokenDenomBySourceResponse {
    pub tokens_denoms: Vec<(String, String, TokenDenom)>,
}

#[cw_serde]
pub struct GetTokenDenomByLptAddressResponse {
    pub tokens_denoms: Vec<(Addr, TokenDenom)>,
}

#[cw_serde]
pub enum MigrateMsg {}

#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct MsgLpTokenMintResponse {}

#[cw_serde]
pub enum Cw20ActionMsg {
    Unstake,
}

#[cw_serde]
pub struct Fee {
    pub amount: String,
    pub recipient: String,
}

#[cw_serde]
pub struct GmpMessage {
    pub destination_chain: String,
    pub destination_address: String,
    pub payload: Binary,
    #[serde(rename = "type")]
    pub type_: i64,
    pub fee: Option<Fee>,
}
