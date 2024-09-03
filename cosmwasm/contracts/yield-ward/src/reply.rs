use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, Reply, Response};

pub fn handle_lp_token_mint_reply(
    _deps: DepsMut,
    _env: Env,
    _msg: Reply,
) -> Result<Response, ContractError> {
    // let id = msg.id;
    // let mint_response = msg
    //     .result
    //     .into_result()
    //     .map_err(ContractError::SubMsgFailure)?
    //     .msg_responses
    //     .iter()
    //     .map(|x| Message::parse_from_bytes(&x.value));

    // .map_err(|err| ContractError::ReplyParseFailure {
    //     id,
    //     err: err.to_string(),
    // })?;

    // unwrap results
    // let trade_data = match order_response.results.into_option() {
    //     Some(trade_data) => Ok(trade_data),
    //     None => Err(ContractError::CustomError {
    //         val: "No trade data in order response".to_string(),
    //     }),
    // }?;
    //
    // let config = STATE.load(deps.storage)?;
    //
    // let cache = SWAP_OPERATION_STATE.load(deps.storage)?;
    //
    // // todo
    //
    // SWAP_OPERATION_STATE.remove(deps.storage);

    unimplemented!()
}
