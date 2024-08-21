use crate::contract::query;
use crate::msg::{
    GetContractConfigResponse, GetStakeStatsResponse, GetTokensConfigsResponse, QueryMsg,
};
use crate::state::{ContractConfigState, StakeStatsItem};
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
                axelar: ctx.axelar,
                lp_token_code_id: 1,
                is_mint_allowed: true,
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

    let tokens_stats_response: GetStakeStatsResponse =
        from_json(query(ctx.deps.as_ref(), ctx.env, QueryMsg::StakeStats).unwrap()).unwrap();
    assert_eq!(
        tokens_stats_response,
        GetStakeStatsResponse {
            stats: ctx
                .tokens
                .iter()
                .map(|(token_denom, _)| (token_denom.clone(), StakeStatsItem::default()))
                .collect()
        }
    );
}
