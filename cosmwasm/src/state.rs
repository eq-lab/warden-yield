use crate::types::{StakeActionStage, TokenConfig, TokenDenom, UnstakeActionStage};
use cosmwasm_std::{Addr, Uint128, Uint256};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONTRACT_CONFIG_STATE: Item<ContractConfigState> = Item::new("contract_config_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractConfigState {
    pub owner: Addr,
    pub axelar: Addr,
}

// Map<> doc - https://book.cosmwasm.com/cross-contract/map-storage.html
pub const TOKENS_CONFIGS_STATE: Map<TokenDenom, TokenConfig> = Map::new("tokens_config_state_map");

pub const TOKENS_STATS_STATE: Map<TokenDenom, TokenStats> = Map::new("tokens_stats_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
pub struct TokenStats {
    pub pending_stake: Uint256,
    pub lp_token_amount: Uint256,
    pub pending_unstake_lp_token_amount: Uint256,
}

pub const STAKE_QUEUE: Map<(TokenDenom, u64), StakeQueueItem> = Map::new("stake_queue");
pub const STAKE_QUEUE_PARAMS: Map<TokenDenom, QueueParams> = Map::new("stake_queue_params");

pub const UNSTAKE_QUEUE: Map<(TokenDenom, u64), UnstakeQueueItem> = Map::new("unstake_queue");
pub const UNSTAKE_QUEUE_PARAMS: Map<TokenDenom, QueueParams> = Map::new("unstake_queue_params");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakeQueueItem {
    pub user: Addr,
    pub token_amount: Uint128,
    pub action_stage: StakeActionStage,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnstakeQueueItem {
    pub user: Addr,
    pub lp_token_amount: Uint128,
    pub action_stage: UnstakeActionStage,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueueParams {
    pub count_active: u64,
    pub end: u64,
}
