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
