use cosmwasm_std::Binary;

pub enum Token {
    WETH,
    USDT,
    USDC,
}

pub struct TokenDetails {
    pub _token: Token,
    pub _lp_token_address: String,
    pub _decimals: u8,
    pub deposit_token_denom: String,
    pub chain: String,
    pub yield_ward_address: String,
    pub evm_yield_contract: String,
}

// todo: actualize after deploy testnet contracts with Bank tokens
pub fn get_token_details(token: &String) -> TokenDetails {
    let chain = "Ethereum".to_string();
    let yield_ward_address =
        "warden1vhjnzk9ly03dugffvzfcwgry4dgc8x0sv0nqqtfxj3ajn7rn5ghq6xwfv9".to_string();

    match token.to_lowercase().as_str() {
        "weth" => TokenDetails {
            _token: Token::WETH,
            deposit_token_denom: "demo_weth".to_string(),
            _decimals: 18,
            _lp_token_address: "warden12nnqks893jx3pz34yj4r4uhlvvgw5e6zjkjwxx03pxtd8y89faqseepx4j"
                .to_string(),
            chain,
            yield_ward_address,
            evm_yield_contract: "0x4DF66BCA96319C6A033cfd86c38BCDb9B3c11a72".to_string(),
        },
        "usdt" => TokenDetails {
            _token: Token::USDT,
            deposit_token_denom: "demo_usdt".to_string(),
            _decimals: 6,
            _lp_token_address: "warden1uyvxsvmjh8j4vnekgdl3rc36pjepaqdqw5999gmkfk05zrs3y03sd6g5wz"
                .to_string(),
            chain,
            yield_ward_address,
            evm_yield_contract: "0x0F9d2C03AD21a30746A4b4f07919e1C5F3641F35".to_string(),
        },
        "usdc" => TokenDetails {
            _token: Token::USDC,
            deposit_token_denom: "demo_usdc".to_string(),
            _decimals: 6,
            _lp_token_address: "warden1qex8hj0ux0vlrzvvnydztlhlrz4whfreahvf49lgjd6zthaf8zsqlq5x4u"
                .to_string(),
            chain,
            yield_ward_address,
            evm_yield_contract: "0x0259044395FE54d8aFe28354Ac737EB216064cF9".to_string(),
        },
        _ => panic!("Unknown token: {}", token),
    }
}

pub fn create_stake_response_payload(
    stake_id: &u64,
    reinit_unstake_id: &u64,
    lp_token_amount: &u128,
    is_success: &bool,
) -> Binary {
    let status = match is_success {
        true => 0_u8,
        false => 1_u8,
    };

    let payload: Vec<u8> = vec![0_u8, status]
        .into_iter()
        .chain(stake_id.to_be_bytes().into_iter())
        .chain(reinit_unstake_id.to_be_bytes().into_iter())
        .chain(lp_token_amount.to_be_bytes().into_iter())
        .collect();

    Binary::new(payload)
}

pub fn create_unstake_response_payload(
    unstake_id: &u64,
    reinit_unstake_id: &u64,
    is_success: &bool,
) -> Binary {
    let status = match is_success {
        true => 0_u8,
        false => 1_u8,
    };

    let payload: Vec<u8> = vec![1_u8, status]
        .into_iter()
        .chain(unstake_id.to_be_bytes().into_iter())
        .chain(reinit_unstake_id.to_be_bytes().into_iter())
        .collect();

    Binary::new(payload)
}
