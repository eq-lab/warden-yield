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

    #[error(
        "Unknown token associated with chain: {source_chain} and source_address: {source_address}"
    )]
    UnknownTokenBySource {
        source_chain: String,
        source_address: String,
    },

    #[error("Unknown LP token: {0}")]
    UnknownLpToken(String),

    #[error("Mismatch CW20 token: {actual}, expected: {expected}")]
    MismatchCw20Token { actual: String, expected: String },

    #[error("Mint is not allowed")]
    MintIsNotAllowed,

    #[error("Message has invalid action type")]
    InvalidActionType,

    #[error("Invalid token: {actual}, expected: {expected}")]
    InvalidToken { actual: String, expected: String },

    #[error("Invalid message payload")]
    InvalidMessagePayload,

    #[error("Token already exist: {0}")]
    TokenAlreadyExist(String),

    #[error("Staking is disabled for token {0}")]
    StakeDisabled(String),

    #[error("Nothing to unstake")]
    NothingToUnstake,

    #[error("Unstake request has invalid stage for {symbol} token, unstake id: {unstake_id}")]
    UnstakeRequestInvalidStage { symbol: String, unstake_id: u64 },

    #[error("Unrecognized reply id: {0}")]
    UnrecognizedReply(u64),

    #[error("Failure response from submsg: {0}")]
    SubMsgFailure(String),

    #[error("Invalid reply from sub-message {id}, {err}")]
    ReplyParseFailure { id: u64, err: String },

    #[error("Zero token amount")]
    ZeroTokenAmount,

    #[error("Invalid CW-20 message")]
    InvalidCw20Message,

    #[error("IBC message failed: {message}")]
    IbcMessageFailed { message: String },

    #[error("IBC sub-message failed: {message}")]
    IbcSubMessageFailed { message: String },

    #[error("IbcOrderedChannel")]
    IbcOrderedChannel,

    #[error("invalid IBC channel version. Got ({actual}), expected ({expected})")]
    InvalidVersion { actual: String, expected: String },

    #[error("Custom Error: {0}")]
    CustomError(String),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
