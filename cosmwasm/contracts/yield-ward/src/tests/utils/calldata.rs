use crate::types::{ReinitResponseData, StakeResponseData, Status, UnstakeResponseData};
use cosmwasm_std::Binary;

pub fn create_stake_response_payload(stake_response_data: StakeResponseData) -> Binary {
    let status = match stake_response_data.status {
        Status::Success => 0_u8,
        Status::Fail => 1_u8,
    };

    let payload: Vec<u8> = vec![0_u8, status]
        .into_iter()
        .chain(stake_response_data.stake_id.to_be_bytes().into_iter())
        .chain(
            stake_response_data
                .reinit_unstake_id
                .to_be_bytes()
                .into_iter(),
        )
        .chain(
            stake_response_data
                .lp_token_amount
                .to_be_bytes()
                .into_iter(),
        )
        .collect();

    Binary::new(payload)
}

pub fn create_unstake_response_payload(unstake_response_data: UnstakeResponseData) -> Binary {
    let status = match unstake_response_data.status {
        Status::Success => 0_u8,
        Status::Fail => 1_u8,
    };

    let payload: Vec<u8> = vec![1_u8, status]
        .into_iter()
        .chain(unstake_response_data.unstake_id.to_be_bytes().into_iter())
        .chain(
            unstake_response_data
                .reinit_unstake_id
                .to_be_bytes()
                .into_iter(),
        )
        .collect();

    Binary::new(payload)
}

pub fn create_reinit_response_payload(reinit_response_data: ReinitResponseData) -> Binary {
    let payload: Vec<u8> = vec![2_u8]
        .into_iter()
        .chain(
            reinit_response_data
                .reinit_unstake_id
                .to_be_bytes()
                .into_iter(),
        )
        .collect();

    Binary::new(payload)
}
