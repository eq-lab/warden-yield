use crate::encoding::{
    decode_payload_action_type, decode_reinit_response_payload, decode_stake_response_payload,
    decode_unstake_response_payload, encode_reinit_payload, encode_stake_payload,
    encode_unstake_payload,
};
use crate::types::{ReinitResponseData, StakeResponseData, Status, UnstakeResponseData};
use cosmwasm_std::{Binary, Uint128};

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
fn test_decode_stake_response_wrong_length() {
    let payload: Vec<u8> = vec![0_u8];
    assert!(decode_stake_response_payload(payload.as_slice()).is_none());
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
fn test_decode_unstake_response_wrong_length() {
    let payload: Vec<u8> = vec![0_u8];
    assert!(decode_unstake_response_payload(payload.as_slice()).is_none());
}

#[test]
fn test_decode_reinit_response() {
    let reinit_unstake_id = 2007_u64; // 8 = 64 bit

    let payload: Vec<u8> = reinit_unstake_id.to_be_bytes().into_iter().collect();

    // println!("Bytes: {:?}", payload);
    let data = decode_reinit_response_payload(payload.as_slice()).unwrap();
    assert_eq!(data, ReinitResponseData { reinit_unstake_id });
}

#[test]
fn test_decode_reinit_response_wrong_length() {
    let payload: Vec<u8> = vec![0_u8];
    assert!(decode_reinit_response_payload(payload.as_slice()).is_none());
}

#[test]
fn test_encode_stake_payload() {
    let stake_id = 5_u64;
    let reinit_payload = encode_stake_payload(stake_id);
    let expected = "0x0000000000000000000000000000000000000000000000000000000000000500";

    assert_eq!(binary_to_hex_string(reinit_payload), expected);
}

#[test]
fn test_encode_unstake_payload() {
    let lp_token_amount = Uint128::from(1000000_u128);
    let unstake_id = 2_u64;

    let reinit_payload = encode_unstake_payload(unstake_id, &lp_token_amount);

    let expected = "0x00000000000000000000000000000000000000000f4240000000000000000201";

    assert_eq!(binary_to_hex_string(reinit_payload), expected);
}

#[test]
fn test_encode_reinit_payload() {
    let reinit_payload = encode_reinit_payload();

    // 0x02 = ActionType::Reinit
    let expected = "0x02";

    assert_eq!(binary_to_hex_string(reinit_payload), expected);
}

fn binary_to_hex_string(arr: Binary) -> String {
    "0x".to_string()
        + arr
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("")
            .as_str()
}

#[test]
fn test_decode_action_type_wrong_byte() {
    let payload: Vec<u8> = vec![3_u8];
    assert!(decode_payload_action_type(&Binary::from(payload)).is_none());
}
