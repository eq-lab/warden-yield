use crate::types::{
    ActionType, ReinitResponseData, StakeResponseData, Status, UnstakeResponseData,
};
use cosmwasm_std::{Binary, Uint128};

pub fn encode_stake_payload(stake_id: u64) -> Binary {
    // 32 bytes:
    //    23 bytes - not used
    //    8 bytes  - StakeId
    //    1 byte   - ActionType
    let payload: Vec<u8> = vec![0_u8; 23]
        .into_iter()
        .chain(stake_id.to_be_bytes().into_iter())
        .chain(Some(0)) // ActionType: stake = 0_u8
        .collect();

    let result: Binary = Binary::new(payload);
    return result;
}

pub fn encode_unstake_payload(unstake_id: u64, lp_token_amount: &Uint128) -> Binary {
    // 32 bytes:
    //    7 bytes - not used
    //    16 bytes - LptAmount
    //    8 bytes  - UnstakeId
    //    1 byte   - ActionType
    let payload: Vec<u8> = vec![0_u8; 7]
        .into_iter()
        .chain(lp_token_amount.to_be_bytes().into_iter())
        .chain(unstake_id.to_be_bytes().into_iter())
        .chain(Some(1)) // ActionType::Unstake = 1_u8
        .collect();

    let result: Binary = Binary::new(payload);
    return result;
}

pub fn encode_reinit_payload() -> Binary {
    let result: Binary = Binary::new(vec![2_u8]);
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
