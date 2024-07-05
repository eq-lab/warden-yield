use cosmwasm_schema::cw_serde;

pub type TokenDenom = String;

#[cw_serde]
pub struct TokenConfig {
    pub is_stake_enabled: bool,
    pub is_unstake_enabled: bool,
    pub symbol: String,
    pub evm_yield_contract: String,
    pub evm_address: String,
    pub lp_token_denom: String,
}

#[cw_serde]
pub enum StakeStatus {
    Successful,
    Fail,
}
