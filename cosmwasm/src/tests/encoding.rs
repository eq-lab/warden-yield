use crate::encoding::{
    decode_reinit_response_payload, decode_stake_response_payload, decode_unstake_response_payload,
};
use crate::types::{ReinitResponseData, StakeResponseData, Status, UnstakeResponseData};
use cosmwasm_std::Uint128;

#[test]
fn test_decode_stake_response() {
    let status = Status::Success; // 1 byte = 8 bit
    let stake_id = 1337_u64; // 8 = 64 bit
    let reinit_unstake_id = 2007_u64; // 8 = 64 bit
    let lp_token_amount = 9000_u128; // 16 = 128 bit

    let payload: Vec<u8> = vec![0_u8]
        .into_iter()
        .chain(stake_id.to_be_bytes().into_iter())
        .chain(reinit_unstake_id.to_be_bytes().into_iter())
        .chain(lp_token_amount.to_be_bytes().into_iter())
        .map(|x| x)
        .collect();

    println!("Bytes: {:?}", payload);
    let data = decode_stake_response_payload(payload.as_slice()).unwrap();
    assert_eq!(
        data,
        StakeResponseData {
            status,
            stake_id,
            reinit_unstake_id,
            lp_token_amount: Uint128::from(lp_token_amount)
        }
    );
}

#[test]
fn test_decode_unstake_response() {
    let status = Status::Success; // 1 byte = 8 bit
    let unstake_id = 1337_u64; // 8 = 64 bit
    let reinit_unstake_id = 2007_u64; // 8 = 64 bit

    let payload: Vec<u8> = vec![0_u8]
        .into_iter()
        .chain(unstake_id.to_be_bytes().into_iter())
        .chain(reinit_unstake_id.to_be_bytes().into_iter())
        .map(|x| x)
        .collect();

    // println!("Bytes: {:?}", payload);
    let data = decode_unstake_response_payload(payload.as_slice()).unwrap();
    assert_eq!(
        data,
        UnstakeResponseData {
            status,
            unstake_id,
            reinit_unstake_id
        }
    );
}

#[test]
fn test_decode_reinit_response() {
    let reinit_unstake_id = 2007_u64; // 8 = 64 bit

    let payload: Vec<u8> = reinit_unstake_id
        .to_be_bytes()
        .into_iter()
        .map(|x| x)
        .collect();

    // println!("Bytes: {:?}", payload);
    let data = decode_reinit_response_payload(payload.as_slice()).unwrap();
    assert_eq!(data, ReinitResponseData { reinit_unstake_id });
}
