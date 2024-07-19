use crate::types::{
    ActionType, ReinitResponseData, StakeResponseData, Status, UnstakeResponseData,
};
use cosmwasm_std::{Binary, Uint128};

pub fn encode_stake_payload(_stake_id: &u64) -> Binary {
    // 8 bits - ActionType: stake = 0, unstake = 1
    // 64 bits - ActionId
    // todo
    // let result: Binary = Binary::new(vec![0_u8, ..stake_id.to_le_bytes()]);
    let result: Binary = Binary::new(vec![]);
    return result;
}

pub fn decode_payload_action_type(payload: &Binary) -> Option<ActionType> {
    match payload.first() {
        Some(0) => Some(ActionType::Stake),
        Some(1) => Some(ActionType::Unstake),
        Some(2) => Some(ActionType::Reinit),
        _ => None,
    }
}

pub fn decode_stake_response_payload(payload: &[u8]) -> Option<StakeResponseData> {
    if payload.len() != 33 {
        return None;
    }

    let status = Status::try_from(payload.first().unwrap()).ok()?;
    let stake_id: u64 = u64::from_be_bytes(payload[1..9].try_into().unwrap());
    let reinit_unstake_id: u64 = u64::from_be_bytes(payload[9..17].try_into().unwrap());
    let lp_token_amount: Uint128 =
        Uint128::from(u128::from_be_bytes(payload[17..33].try_into().unwrap()));

    Some(StakeResponseData {
        status,
        stake_id,
        reinit_unstake_id,
        lp_token_amount,
    })
}

pub fn decode_unstake_response_payload(payload: &[u8]) -> Option<UnstakeResponseData> {
    if payload.len() != 17 {
        return None;
    }

    let status = Status::try_from(payload.first().unwrap()).ok()?;
    let unstake_id: u64 = u64::from_be_bytes(payload[1..9].try_into().unwrap());
    let reinit_unstake_id: u64 = u64::from_be_bytes(payload[9..17].try_into().unwrap());

    Some(UnstakeResponseData {
        status,
        unstake_id,
        reinit_unstake_id,
    })
}

pub fn decode_reinit_response_payload(payload: &[u8]) -> Option<ReinitResponseData> {
    if payload.len() != 8 {
        return None;
    }
    let reinit_unstake_id: u64 = u64::from_be_bytes(payload.try_into().unwrap());
    Some(ReinitResponseData { reinit_unstake_id })
}
