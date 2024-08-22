use crate::state::{ContractConfigState, StakeStatsItem};
use crate::tests::utils::init::instantiate_yield_ward_contract_without_tokens;
use crate::tests::utils::query::{
    get_all_stake_stats, get_all_tokens_configs, get_contract_config,
};

#[test]
fn test_instantiate() {
    let (app, ctx) = instantiate_yield_ward_contract_without_tokens();

    let config_response = get_contract_config(&app, &ctx);

    assert_eq!(
        config_response,
        ContractConfigState {
            owner: ctx.admin.clone(),
            axelar: ctx.axelar.clone(),
            lp_token_code_id: 1,
            is_mint_allowed: true,
        }
    );

    let tokens_configs = get_all_tokens_configs(&app, &ctx);
    assert!(tokens_configs.is_empty());

    let tokens_stats = get_all_stake_stats(&app, &ctx);
    assert_eq!(
        tokens_stats,
        ctx.tokens
            .iter()
            .map(|token_info| (
                token_info.deposit_token_denom.clone(),
                StakeStatsItem::default()
            ))
            .collect()
    );
}
