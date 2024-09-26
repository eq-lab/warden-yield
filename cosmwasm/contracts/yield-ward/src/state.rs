use crate::types::{StakeActionStage, TokenConfig, TokenDenom, UnstakeActionStage};
use cosmwasm_std::{Addr, Uint128, Uint256};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONTRACT_CONFIG: Item<ContractConfigState> = Item::new("contract_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractConfigState {
    pub owner: Addr,
    pub axelar: Addr,
    pub lp_token_code_id: u64,
    pub is_mint_allowed: bool,
}

pub const AXELAR_CONFIG: Item<AxelarConfigState> = Item::new("axelar_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AxelarConfigState {
    pub axelar_channel_id: String,
    pub axelar_gateway_cosmos_address: String,
    pub axelar_fee_recipient_address: String,
    pub ibc_timeout_seconds: u64,
}

// Map<> doc - https://book.cosmwasm.com/cross-contract/map-storage.html
pub const TOKEN_CONFIG: Map<&TokenDenom, TokenConfig> = Map::new("token_config_map");

/// Map<(source_chain, source_address), token_denom>
pub const TOKEN_DENOM_BY_SOURCE: Map<(&str, &str), TokenDenom> =
    Map::new("token_denom_by_source_map");

pub const TOKEN_DENOM_BY_LPT_ADDRESS: Map<&Addr, TokenDenom> =
    Map::new("token_denom_by_lpt_address");

pub const STAKE_STATS: Map<&TokenDenom, StakeStatsItem> = Map::new("stake_stats_map");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
pub struct StakeStatsItem {
    pub pending_stake: Uint256,
    pub lp_token_amount: Uint256,
    pub pending_unstake_lp_token_amount: Uint256,
}

pub const STAKES: Map<(&TokenDenom, u64), StakeItem> = Map::new("stakes_map");
pub const STAKE_PARAMS: Map<&TokenDenom, QueueParams> = Map::new("stake_params");

pub const UNSTAKES: Map<(&TokenDenom, u64), UnstakeItem> = Map::new("unstakes_map");
pub const UNSTAKE_PARAMS: Map<&TokenDenom, QueueParams> = Map::new("unstake_params");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakeItem {
    pub user: Addr,
    pub token_amount: Uint128,
    pub action_stage: StakeActionStage,
    pub lp_token_amount: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnstakeItem {
    pub user: Addr,
    pub lp_token_amount: Uint128,
    pub action_stage: UnstakeActionStage,
    pub token_amount: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueueParams {
    /// Count of stake/unstake requests in pending state
    pub pending_count: u64,
    /// Id counter for stake, unstake requests
    pub next_id: u64,
}
