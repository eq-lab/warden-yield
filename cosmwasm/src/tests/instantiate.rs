use crate::contract::query;
use crate::msg::{
    GetContractConfigResponse, GetTokensConfigsResponse, GetTokensStatsResponse, QueryMsg,
};
use crate::state::{ContractConfigState, TokenStats};
use crate::tests::utils::instantiate_contract;
use cosmwasm_std::from_json;

#[test]
fn test_instantiate() {
    let ctx = instantiate_contract();

    let config_response: GetContractConfigResponse =
        from_json(query(ctx.deps.as_ref(), ctx.env.clone(), QueryMsg::ContractConfig).unwrap())
            .unwrap();
    assert_eq!(
        config_response,
        GetContractConfigResponse {
            config: ContractConfigState {
                owner: ctx.admin,
                axelar: ctx.axelar
            }
        }
    );

    let tokens_configs_response: GetTokensConfigsResponse =
        from_json(query(ctx.deps.as_ref(), ctx.env.clone(), QueryMsg::TokensConfigs).unwrap())
            .unwrap();
    assert_eq!(
        tokens_configs_response,
        GetTokensConfigsResponse {
            tokens: ctx.tokens.clone()
        }
    );

    let tokens_stats_response: GetTokensStatsResponse =
        from_json(query(ctx.deps.as_ref(), ctx.env, QueryMsg::TokensStats).unwrap()).unwrap();
    assert_eq!(
        tokens_stats_response,
        GetTokensStatsResponse {
            stats: ctx
                .tokens
                .iter()
                .map(|(token_denom, _)| (token_denom.clone(), TokenStats::default()))
                .collect()
        }
    );
}
