use cosmwasm_std::Binary;

pub enum Token {
    WETH,
    USDT,
    USDC,
}

pub struct TokenDetails {
    pub _token: Token,
    pub _deposit_token_address: String,
    pub _lp_token_address: String,
    pub deposit_token_denom: String,
    pub chain: String,
    pub yield_ward_address: String,
    pub evm_yield_contract: String,
}

pub fn get_token_details(token: &String) -> TokenDetails {
    let chain = "Ethereum".to_string();
    let yield_ward_address =
        "warden1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0say3p32".to_string();

    match token.to_lowercase().as_str() {
        "weth" => TokenDetails {
            _token: Token::WETH,
            deposit_token_denom: "demo_weth".to_string(),
            _deposit_token_address:
                "warden1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qvhthj6".to_string(),
            _lp_token_address: "warden1fhlq0karyxn8jp0ky9rsckdzu6n60uge3cpd6vftj627ffmma5zsjtnr5h"
                .to_string(),
            chain,
            yield_ward_address,
            evm_yield_contract: "0x4DF66BCA96319C6A033cfd86c38BCDb9B3c11a72".to_string(),
        },
        "usdt" => TokenDetails {
            _token: Token::USDT,
            deposit_token_denom: "demo_usdt".to_string(),
            _deposit_token_address:
                "warden1qwlgtx52gsdu7dtp0cekka5zehdl0uj3fhp9acg325fvgs8jdzkssm2jq5".to_string(),
            _lp_token_address: "warden1vssdwz32692ph7r6u28l8rjuap8jyce02ng40c37rjwh3vcqal5s32era6"
                .to_string(),
            chain,
            yield_ward_address,
            evm_yield_contract: "0x0F9d2C03AD21a30746A4b4f07919e1C5F3641F35".to_string(),
        },
        "usdc" => TokenDetails {
            _token: Token::USDC,
            deposit_token_denom: "demo_usdc".to_string(),
            _deposit_token_address:
                "warden1fzm6gzyccl8jvdv3qq6hp9vs6ylaruervs4m06c7k0ntzn2f8faqnje7c3".to_string(),
            _lp_token_address: "warden1u8ct0fkqahwe38qe09yqjwcdxajfe2a9cg2tltvk2vhc7hxuz78q6umxq5"
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
