mod common;

use crate::common::{
    create_stake_response_payload, create_unstake_response_payload, get_token_details,
};
use cosmwasm_std::{to_json_binary, to_json_string, Uint128};
use std::string::ToString;
use yield_ward::msg::Cw20ActionMsg;

use clap::{command, Parser, Subcommand};
use cw20::Cw20ExecuteMsg;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Stake {
        #[arg(long)]
        token: String,
    },
    StakeResponse {
        #[arg(long)]
        token: String,
        #[arg(long)]
        stake_id: u64,
        #[arg(long)]
        reinit_unstake_id: u64,
        #[arg(long)]
        lp_token_amount: u128,
        #[arg(long)]
        is_success: bool,
        #[arg(long)]
        return_amount: u128,
    },
    Unstake {
        #[arg(long)]
        token: String,
    },
    UnstakeResponse {
        #[arg(long)]
        token: String,
        #[arg(long)]
        unstake_id: u64,
        #[arg(long)]
        reinit_unstake_id: u64,
        #[arg(long)]
        is_success: bool,
        #[arg(long)]
        return_amount: u128,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Stake { token } => process_stake(token),
        Commands::StakeResponse {
            token,
            stake_id,
            reinit_unstake_id,
            lp_token_amount,
            is_success,
            return_amount,
        } => process_stake_response(
            token,
            stake_id,
            reinit_unstake_id,
            lp_token_amount,
            is_success,
            return_amount,
        ),
        Commands::Unstake { token } => process_unstake(token),
        Commands::UnstakeResponse {
            token,
            unstake_id,
            reinit_unstake_id,
            is_success,
            return_amount,
        } => process_unstake_response(
            token,
            unstake_id,
            reinit_unstake_id,
            is_success,
            return_amount,
        ),
    }
}

fn process_stake(token: &String) {
    let token_details = get_token_details(&token);
    let encoded = to_json_binary(&Cw20ActionMsg::Stake {
        deposit_token_denom: token_details.deposit_token_denom.clone(),
    })
    .unwrap();

    println!("Cw20ActionMsg::Stake inner call hex: {}", encoded);

    let result = to_json_string(&Cw20ExecuteMsg::Send {
        contract: token_details.yield_ward_address,
        amount: Uint128::new(12345),
        msg: encoded,
    })
    .unwrap();
    println!("Encoded stake message: {}", result);
}

fn process_stake_response(
    token: &String,
    stake_id: &u64,
    reinit_unstake_id: &u64,
    lp_token_amount: &u128,
    is_success: &bool,
    return_amount: &u128,
) {
    println!("token: {}, stake_id: {}, reinit_unstake_id: {}, lp_token_amount: {}, is_success: {}, return_amount: {}",
        token, stake_id, reinit_unstake_id,
        lp_token_amount, is_success, return_amount);

    let token_details = get_token_details(&token);
    let response_payload =
        create_stake_response_payload(stake_id, reinit_unstake_id, lp_token_amount, is_success);

    println!("Response payload hex: {:?}", response_payload);

    let encoded = to_json_binary(&Cw20ActionMsg::HandleResponse {
        deposit_token_denom: token_details.deposit_token_denom.clone(),
        source_chain: token_details.chain.to_string(),
        source_address: token_details.evm_yield_contract.to_string(),
        payload: response_payload,
    })
    .unwrap();
    println!("Cw20ActionMsg::Response inner call hex: {}", encoded);

    let result = to_json_string(&Cw20ExecuteMsg::Send {
        contract: token_details.yield_ward_address.to_string(),
        amount: Uint128::new(*return_amount),
        msg: encoded,
    })
    .unwrap();

    println!("Encoded stake response message: {}", result);
}

fn process_unstake(token: &String) {
    let token_details = get_token_details(&token);
    let encoded = to_json_binary(&Cw20ActionMsg::Unstake {
        deposit_token_denom: token_details.deposit_token_denom.clone(),
    })
    .unwrap();

    let result = to_json_string(&Cw20ExecuteMsg::Send {
        contract: token_details.yield_ward_address,
        amount: Uint128::new(123),
        msg: encoded,
    })
    .unwrap();
    println!("Encoded unstake message: {}", result);
}

fn process_unstake_response(
    token: &String,
    unstake_id: &u64,
    reinit_unstake_id: &u64,
    is_success: &bool,
    return_amount: &u128,
) {
    println!(
        "token: {}, unstake_id: {}, reinit_unstake_id: {}, is_success: {}, return_amount: {}",
        token, unstake_id, reinit_unstake_id, is_success, return_amount
    );

    let token_details = get_token_details(&token);
    let response_payload =
        create_unstake_response_payload(unstake_id, reinit_unstake_id, is_success);

    println!("Response payload hex: {:?}", response_payload);

    let encoded = to_json_binary(&Cw20ActionMsg::HandleResponse {
        deposit_token_denom: token_details.deposit_token_denom.clone(),
        source_chain: token_details.chain.to_string(),
        source_address: token_details.evm_yield_contract.to_string(),
        payload: response_payload,
    })
    .unwrap();
    println!("Cw20ActionMsg::Response inner call hex: {}", encoded);

    let result = to_json_string(&Cw20ExecuteMsg::Send {
        contract: token_details.yield_ward_address.to_string(),
        amount: Uint128::new(*return_amount),
        msg: encoded,
    })
    .unwrap();

    println!("Encoded unstake response message: {}", result);
}
