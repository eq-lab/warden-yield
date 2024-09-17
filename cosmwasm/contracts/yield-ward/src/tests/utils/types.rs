use crate::types::TokenDenom;
use cosmwasm_std::{Addr, Empty, Uint128};
use cw_multi_test::{App, FailingModule};

pub type TestingApp<ExecC = Empty, QueryC = Empty> = App<
    cw_multi_test::BankKeeper,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockStorage,
    cw_multi_test::FailingModule<ExecC, QueryC, Empty>,
    cw_multi_test::WasmKeeper<ExecC, QueryC>,
    FailingModule<Empty, Empty, Empty>,
    FailingModule<Empty, Empty, Empty>,
    cw_multi_test::IbcAcceptingModule,
    cw_multi_test::GovFailingModule,
    cw_multi_test::StargateFailing,
>;

pub struct TestInfo {
    pub lp_token_code_id: u64,
    pub yield_ward_address: Addr,
    pub admin: Addr,
    pub user: Addr,
    pub unstake_user: Addr,
    pub axelar: Addr,
    pub tokens: Vec<TokenTestInfo>,
}

pub struct TokenTestInfo {
    pub deposit_token_denom: TokenDenom,
    pub deposit_token_symbol: String,
    pub deposit_token_decimals: u8,
    pub is_stake_enabled: bool,
    pub is_unstake_enabled: bool,
    pub symbol: String,
    pub name: String,
    pub chain: String,
    pub evm_yield_contract: String,
    pub evm_address: String,
}

pub struct UnstakeDetails {
    pub _stake_id: u64,
    pub _stake_amount: Uint128,
    pub unstake_id: u64,
    pub lp_token_amount: Uint128,
    pub unstake_token_amount: Uint128,
}
