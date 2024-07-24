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
}

// Map<> doc - https://book.cosmwasm.com/cross-contract/map-storage.html
pub const TOKEN_CONFIG: Map<&TokenDenom, TokenConfig> = Map::new("token_config_map");

/// Map<(source_chain, source_address), token_denom>
pub const TOKEN_DENOM_BY_SOURCE: Map<(&String, &String), TokenDenom> =
    Map::new("token_denom_by_source_map");

pub const TOKEN_STATS: Map<&TokenDenom, StakeStatsItem> = Map::new("stake_stats_map");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
pub struct StakeStatsItem {
    pub pending_stake: Uint256,
    pub lp_token_amount: Uint256,
    pub pending_unstake_lp_token_amount: Uint256,
}

pub const STAKES: Map<(&TokenDenom, u64), StakeItem> = Map::new("stakes_map");
pub const STAKE_QUEUE_PARAMS: Map<&TokenDenom, QueueParams> = Map::new("stake_queue_params");

pub const UNSTAKES: Map<(&TokenDenom, u64), UnstakeItem> = Map::new("unstakes_map");
pub const UNSTAKE_QUEUE_PARAMS: Map<&TokenDenom, QueueParams> = Map::new("unstake_queue_params");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakeItem {
    pub user: Addr,
    pub token_amount: Uint128,
    pub action_stage: StakeActionStage,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnstakeItem {
    pub user: Addr,
    pub lp_token_amount: Uint128,
    pub action_stage: UnstakeActionStage,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueueParams {
    /// Count of stake/unstake requests in pending state
    pub pending_count: u64,
    /// Id counter for stake, unstake requests
    pub next_id: u64,
}
