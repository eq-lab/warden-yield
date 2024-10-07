use std::marker::PhantomData;

use crate::msg::{GmpMessage, GmpMsgType};
use cosmwasm_std::{
    Addr, Api, Binary, BlockInfo, Empty, IbcMsg, IbcQuery, Querier, Storage, Uint128,
};
use cw_multi_test::{
    error::{anyhow, bail, AnyResult},
    AppResponse, CosmosRouter, Ibc, Module,
};
use serde_json_wasm::from_str;

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
        msg: Self::ExecT,
    ) -> AnyResult<AppResponse> {
        let (channel_id, to_address, amount, timeout, memo) = match msg {
            IbcMsg::Transfer {
                channel_id,
                to_address,
                amount,
                timeout,
                memo,
            } => (channel_id, to_address, amount, timeout, memo),
            _ => bail!("Wrong IbcMsg type"),
        };

        assert_eq!(
            channel_id,
            crate::tests::utils::constants::AXELAR_CHANNEL_ID
        );
        assert_eq!(
            to_address,
            crate::tests::utils::constants::AXELAR_GATEWAY_COSMOS_ADDRESS
        );
        assert_eq!(
            timeout,
            block
                .time
                .plus_seconds(crate::tests::utils::constants::IBC_TIMEOUT_SECONDS)
                .into()
        );

        let ibc_memo = memo.ok_or_else(|| anyhow!("Memo can't be None"))?;
        let gmp_message = from_str::<GmpMessage>(&ibc_memo)
            .map_err(|_| anyhow!("Failed to deserialize {:?} memo", ibc_memo))?;

        assert_eq!(
            gmp_message.destination_chain,
            crate::tests::utils::constants::EVM_DESTINATION_CHAIN_TAG
        );
        assert_eq!(
            gmp_message.destination_address,
            crate::tests::utils::constants::YIELD_WARD_EVM_ADDRESS
        );

        assert!(gmp_message.fee.is_some());

        let fee = gmp_message.fee.unwrap();
        assert_eq!(
            fee.recipient,
            crate::tests::utils::constants::AXELAR_FEE_RECIPIENT_ADDRESS
        );

        let fee_amount: Uint128 = fee
            .amount
            .trim()
            .parse::<u128>()
            .map_err(|_| anyhow!("Failed to parse stake fee amount"))?
            .into();

        match gmp_message
            .payload
            .last()
            .ok_or_else(|| anyhow!("Empty gmp message payload"))?
        {
            0 => {
                // stake
                assert_eq!(gmp_message.type_, GmpMsgType::WithToken as i64);
                assert!(!amount.amount.is_zero());
                assert!(!fee_amount.is_zero());
                assert!(fee_amount < amount.amount);
            }
            1 => {
                // unstake
                assert_eq!(gmp_message.type_, GmpMsgType::Pure as i64);
                assert_eq!(
                    amount.amount,
                    Uint128::from(crate::tests::utils::constants::AXELAR_FEE)
                );
                assert_eq!(fee_amount, amount.amount);
            }
            2 => {
                // reinit
                assert_eq!(gmp_message.type_, GmpMsgType::Pure as i64);
                assert_eq!(
                    amount.amount,
                    Uint128::from(crate::tests::utils::constants::AXELAR_FEE)
                );
                assert_eq!(fee_amount, amount.amount);
            }
            _ => bail!("Wrong action type"),
        };

        Ok(AppResponse::default())
    }

    /// Runs any [QueryT](Self::QueryT) message, always returns a default (empty) binary.
    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: Self::QueryT,
    ) -> AnyResult<Binary> {
        bail!("Unexpected custom query {:?}", request)
    }

    /// Runs any [SudoT](Self::SudoT) privileged action, always returns a default response.
    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        msg: Self::SudoT,
    ) -> AnyResult<AppResponse> {
        bail!("Unexpected sudo msg {:?}", msg)
    }
}

impl Ibc for IbcModuleMock {}
