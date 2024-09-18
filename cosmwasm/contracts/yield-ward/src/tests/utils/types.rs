use std::marker::PhantomData;

use crate::{
    msg::{GmpMessage, GmpMsgType},
    types::{ActionType, TokenDenom},
};
use cosmwasm_std::{
    Addr, Api, Binary, BlockInfo, Empty, IbcMsg, IbcQuery, Querier, Storage, Uint128,
};
use cw_multi_test::{
    error::{anyhow, AnyResult},
    App, AppResponse, CosmosRouter, FailingModule, Ibc, Module,
};
use serde_json_wasm::from_str;

pub type TestingApp<ExecC = Empty, QueryC = Empty> = App<
    cw_multi_test::BankKeeper,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockStorage,
    cw_multi_test::FailingModule<ExecC, QueryC, Empty>,
    cw_multi_test::WasmKeeper<ExecC, QueryC>,
    FailingModule<Empty, Empty, Empty>,
    FailingModule<Empty, Empty, Empty>,
    IbcModuleMock,
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

pub struct IbcModuleMock(PhantomData<Empty>);

impl IbcModuleMock {
    /// Creates an instance of an accepting module.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl Default for IbcModuleMock {
    /// Creates an instance of an accepting module with default settings.
    fn default() -> Self {
        Self::new()
    }
}

impl Module for IbcModuleMock {
    type ExecT = IbcMsg;
    type QueryT = IbcQuery;
    type SudoT = Empty;

    /// Runs any [ExecT](Self::ExecT) message, always returns a default response.
    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        _sender: Addr,
        _msg: Self::ExecT,
    ) -> AnyResult<AppResponse> {
        let (channel_id, to_address, amount, timeout, memo) = match _msg {
            IbcMsg::Transfer {
                channel_id,
                to_address,
                amount,
                timeout,
                memo,
            } => (channel_id, to_address, amount, timeout, memo),
            _ => return Err(anyhow!("Wrong IbcMsg type")),
        };

        assert_eq!(channel_id, super::constants::AXELAR_CHANNEL_ID);
        assert_eq!(to_address, super::constants::AXELAR_GATEWAY_COSMOS_ADDRESS);
        assert_eq!(
            timeout,
            block
                .time
                .plus_seconds(super::constants::IBC_TIMEOUT_SECONDS)
                .into()
        );

        let ibc_memo = memo.ok_or_else(|| anyhow!("Memo can't be None"))?;
        let gmp_message = from_str::<GmpMessage>(&ibc_memo)
            .map_err(|_| anyhow!("Failed to deserialize {:?} memo", ibc_memo))?;

        assert_eq!(
            gmp_message.destination_chain,
            super::constants::EVM_DESTINATION_CHAIN_TAG
        );
        assert_eq!(
            gmp_message.destination_address,
            super::constants::YIELD_WARD_EVM_ADDRESS
        );

        let action_type = match gmp_message
            .payload
            .last()
            .ok_or_else(|| anyhow!("Empty gmp message payload"))?
        {
            0 => ActionType::Stake,
            1 => ActionType::Unstake,
            2 => ActionType::Reinit,
            _ => return Err(anyhow!("Wrong action type")),
        };

        match action_type {
            ActionType::Stake => {
                assert_eq!(gmp_message.type_, GmpMsgType::WithToken as i64);
                assert!(!amount.amount.is_zero());
                assert!(gmp_message.fee.is_none());
            }
            ActionType::Unstake => {
                assert_eq!(gmp_message.type_, GmpMsgType::Pure as i64);
                assert!(amount.amount.is_zero());
                assert!(gmp_message.fee.is_none());
            }
            ActionType::Reinit => {
                assert_eq!(gmp_message.type_, GmpMsgType::Pure as i64);
                assert!(amount.amount.is_zero());
                assert!(gmp_message.fee.is_none());
            }
        }

        Ok(AppResponse::default())
    }

    /// Runs any [QueryT](Self::QueryT) message, always returns a default (empty) binary.
    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        _request: Self::QueryT,
    ) -> AnyResult<Binary> {
        Ok(Binary::default())
    }

    /// Runs any [SudoT](Self::SudoT) privileged action, always returns a default response.
    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _msg: Self::SudoT,
    ) -> AnyResult<AppResponse> {
        Ok(AppResponse::default())
    }
}

impl Ibc for IbcModuleMock {}
