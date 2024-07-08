use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Unknown token: {0}")]
    UnknownToken(String),

    #[error("Unknown LP token: {0}")]
    UnknownLpToken(String),

    #[error("Invalid token: {actual}, expected: {expected}")]
    InvalidToken { actual: String, expected: String },

    #[error("Token already exist: {0}")]
    TokenAlreadyExist(String),

    #[error("Staking is disabled for token {0}")]
    StakeDisabled(String),

    #[error("Nothing to unstake")]
    NothingToUnstake,

    #[error("Unrecognized reply id: {0}")]
    UnrecognizedReply(u64),

    #[error("Failure response from submsg: {0}")]
    SubMsgFailure(String),

    #[error("Invalid reply from sub-message {id}, {err}")]
    ReplyParseFailure { id: u64, err: String },

    #[error("Custom Error: {0}")]
    CustomError(String),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
