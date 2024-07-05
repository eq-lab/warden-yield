use crate::types::{TokenConfig, TokenDenom};
use cosmwasm_std::{Addr, Uint128};
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
pub const USERS_STATS_STATE: Map<(Addr, TokenDenom), TokenStats> = Map::new("users_stats_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TokenStats {
    pub pending_stake: Uint128,
    pub staked_shares_amount: Uint128,
    pub pending_shares_unstake: Uint128,
}
