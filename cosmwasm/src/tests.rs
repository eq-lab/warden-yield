#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        ExecuteMsg, GetContractConfigResponse, GetTokensConfigsResponse, GetTokensStatsResponse,
        GetUserStatsResponse, InstantiateMsg, QueryMsg,
    };
    use crate::state::{ContractConfigState, TokenStats};
    use crate::types::{StakeStatus, TokenConfig, TokenDenom};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{from_json, Addr, Coin, Deps, Env, OwnedDeps, Uint128};
    use std::collections::HashMap;

    struct TestContext {
        pub deps: OwnedDeps<MockStorage, MockApi, MockQuerier>,
        pub env: Env,
        pub admin: Addr,
        pub user: Addr,
        pub axelar: Addr,
        pub tokens: Vec<(TokenDenom, TokenConfig)>,
    }

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
                    .map(|(token_denom, _)| (
                        token_denom.clone(),
                        TokenStats {
                            pending_stake: Uint128::zero(),
                            staked_shares_amount: Uint128::zero(),
                            pending_shares_unstake: Uint128::zero()
                        }
                    ))
                    .collect()
            }
        );
    }

    #[test]
    fn test_init_stake_one_coin() {
        let mut ctx = instantiate_contract();
        let stake_amount = Uint128::from(1000_u32);
        let (token_denom, _) = ctx.tokens.first().clone().unwrap();

        // init stake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(
                &ctx.user.clone(),
                &vec![Coin {
                    denom: token_denom.clone(),
                    amount: stake_amount,
                }],
            ),
            ExecuteMsg::Stake,
        )
        .unwrap();

        // check stats
        let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);

        assert_eq!(token_stats.pending_stake, stake_amount);
        assert_eq!(token_stats.staked_shares_amount, Uint128::zero());
        assert_eq!(token_stats.pending_shares_unstake, Uint128::zero());

        let user_stats = get_user_token_stats(
            ctx.deps.as_ref(),
            ctx.env.clone(),
            ctx.user.clone(),
            token_denom.clone(),
        );
        assert_eq!(user_stats.pending_stake, stake_amount);
        assert_eq!(user_stats.staked_shares_amount, Uint128::zero());
        assert_eq!(user_stats.pending_shares_unstake, Uint128::zero());
    }

    #[test]
    fn test_stake_multiple_coins_in_one_tx() {
        let mut ctx = instantiate_contract();
        let stake_amounts: Vec<Coin> = ctx
            .tokens
            .iter()
            .enumerate()
            .map(|(i, (token_denom, _))| Coin {
                denom: token_denom.clone(),
                amount: Uint128::from((i as u32 + 1_u32) * 1000),
            })
            .collect();

        // init stake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.user.clone(), &stake_amounts),
            ExecuteMsg::Stake,
        )
        .unwrap();

        // check stats
        let tokens_stats = get_tokens_stats(ctx.deps.as_ref(), ctx.env.clone());
        let user_tokens_stats = get_user_tokens_stats(
            ctx.deps.as_ref(),
            ctx.env.clone(),
            ctx.user.clone(),
            &ctx.tokens,
        );

        for coin in &stake_amounts {
            let token_denom = &coin.denom;
            let token_stats = &tokens_stats[token_denom];
            let user_stats = &user_tokens_stats[token_denom];

            assert_eq!(token_stats.pending_stake, coin.amount);
            assert_eq!(token_stats.staked_shares_amount, Uint128::zero());
            assert_eq!(token_stats.pending_shares_unstake, Uint128::zero());

            assert_eq!(user_stats.pending_stake, coin.amount);
            assert_eq!(user_stats.staked_shares_amount, Uint128::zero());
            assert_eq!(user_stats.pending_shares_unstake, Uint128::zero());
        }
    }

    #[test]
    fn test_stake_in_two_tx() {
        let mut ctx = instantiate_contract();

        let (token_denom, _) = ctx.tokens.first().clone().unwrap();
        let stake_amount_1 = Uint128::from(1000_u32);
        let stake_amount_2 = Uint128::from(2000_u32);
        let staked_total = stake_amount_1 + stake_amount_2;

        // init first stake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(
                &ctx.user.clone(),
                &vec![Coin {
                    denom: token_denom.clone(),
                    amount: stake_amount_1,
                }],
            ),
            ExecuteMsg::Stake,
        )
        .unwrap();

        // init second stake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(
                &ctx.user.clone(),
                &vec![Coin {
                    denom: token_denom.clone(),
                    amount: stake_amount_2,
                }],
            ),
            ExecuteMsg::Stake,
        )
        .unwrap();

        // check stats
        let token_stats = get_token_stats(ctx.deps.as_ref(), ctx.env.clone(), &token_denom);

        assert_eq!(token_stats.pending_stake, staked_total);
        assert_eq!(token_stats.staked_shares_amount, Uint128::zero());
        assert_eq!(token_stats.pending_shares_unstake, Uint128::zero());

        let user_stats = get_user_token_stats(
            ctx.deps.as_ref(),
            ctx.env.clone(),
            ctx.user.clone(),
            token_denom.clone(),
        );

        assert_eq!(user_stats.pending_stake, staked_total);
        assert_eq!(user_stats.staked_shares_amount, Uint128::zero());
        assert_eq!(user_stats.pending_shares_unstake, Uint128::zero());
    }

    #[test]
    fn test_stake_response_successful() {
        let mut ctx = instantiate_contract();
        let (token_denom, token_config) = ctx.tokens.first().clone().unwrap();
        let stake_amount = Uint128::from(1000_u32);
        let shares_amount = stake_amount + Uint128::one();

        // init stake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(
                &ctx.user.clone(),
                &vec![Coin {
                    amount: stake_amount,
                    denom: token_denom.clone(),
                }],
            ),
            ExecuteMsg::Stake,
        )
        .unwrap();

        // response for stake action
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.axelar.clone(), &vec![]),
            ExecuteMsg::HandleStakeResponse {
                account: ctx.user.clone(),
                token_evm: token_config.evm_address.clone(),
                token_amount: stake_amount,
                shares_amount,
                status: StakeStatus::Successful,
            },
        )
        .unwrap();

        // check stats
        let tokens_stats = get_tokens_stats(ctx.deps.as_ref(), ctx.env.clone());
        let token_stats = &tokens_stats[token_denom];

        assert_eq!(token_stats.pending_stake, Uint128::zero());
        assert_eq!(token_stats.staked_shares_amount, shares_amount);
        assert_eq!(token_stats.pending_shares_unstake, Uint128::zero());

        let user_tokens_stats = get_user_tokens_stats(
            ctx.deps.as_ref(),
            ctx.env.clone(),
            ctx.user.clone(),
            &ctx.tokens,
        );

        let user_stats = &user_tokens_stats[token_denom];
        assert_eq!(user_stats.pending_stake, Uint128::zero());
        assert_eq!(user_stats.staked_shares_amount, shares_amount);
        assert_eq!(user_stats.pending_shares_unstake, Uint128::zero());

        // todo: check LP tokens are minted
    }

    #[test]
    fn test_stake_response_fail() {
        let mut ctx = instantiate_contract();
        let stake_amount = Uint128::from(1000_u32);
        let shares_amount = stake_amount + Uint128::one();
        let (token_denom, token_config) = ctx.tokens.first().clone().unwrap();
        let stake_funds = vec![Coin {
            amount: stake_amount,
            denom: token_denom.clone(),
        }];

        // init stake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.user.clone(), &stake_funds),
            ExecuteMsg::Stake,
        )
        .unwrap();

        // response for stake action
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.axelar.clone(), &stake_funds),
            ExecuteMsg::HandleStakeResponse {
                account: ctx.user.clone(),
                token_evm: token_config.evm_address.clone(),
                shares_amount,
                token_amount: stake_amount,
                status: StakeStatus::Fail,
            },
        )
        .unwrap();

        // check stats
        let tokens_stats = get_tokens_stats(ctx.deps.as_ref(), ctx.env.clone());
        let token_stats = &tokens_stats[token_denom];

        assert_eq!(token_stats.pending_stake, Uint128::zero());
        assert_eq!(token_stats.staked_shares_amount, Uint128::zero());
        assert_eq!(token_stats.pending_shares_unstake, Uint128::zero());

        let user_tokens_stats = get_user_tokens_stats(
            ctx.deps.as_ref(),
            ctx.env.clone(),
            ctx.user.clone(),
            &ctx.tokens,
        );

        let user_stats = &user_tokens_stats[token_denom];
        assert_eq!(user_stats.pending_stake, Uint128::zero());
        assert_eq!(user_stats.staked_shares_amount, Uint128::zero());
        assert_eq!(user_stats.pending_shares_unstake, Uint128::zero());

        // todo: check LP tokens are not minted
        // todo: check deposited tokens are returned to user
    }

    #[test]
    fn test_unstake_response_successful() {
        let mut ctx = instantiate_contract();
        let (token_denom, token_config) = ctx.tokens.first().clone().unwrap();
        let stake_amount = Uint128::from(1000_u32);
        let shares_amount = stake_amount + Uint128::one();
        let return_amount = shares_amount + Uint128::one();

        // init stake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(
                &ctx.user.clone(),
                &vec![Coin {
                    amount: stake_amount,
                    denom: token_denom.clone(),
                }],
            ),
            ExecuteMsg::Stake,
        )
        .unwrap();

        // response for stake action
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.axelar.clone(), &vec![]),
            ExecuteMsg::HandleStakeResponse {
                account: ctx.user.clone(),
                token_evm: token_config.evm_address.clone(),
                token_amount: stake_amount,
                shares_amount,
                status: StakeStatus::Successful,
            },
        )
        .unwrap();

        // init unstake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.user.clone(), &vec![]),
            ExecuteMsg::Unstake {
                token_denom: token_denom.clone(),
            },
        )
        .unwrap();

        // response for unstake action
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(
                &ctx.axelar.clone(),
                &vec![Coin {
                    denom: token_denom.clone(),
                    amount: return_amount,
                }],
            ),
            ExecuteMsg::HandleUnstakeResponse {
                account: ctx.user.clone(),
                token_evm: token_config.evm_address.clone(),
                token_amount: return_amount,
                shares_amount,
                status: StakeStatus::Successful,
            },
        )
        .unwrap();

        // check stats
        let tokens_stats = get_tokens_stats(ctx.deps.as_ref(), ctx.env.clone());
        let token_stats = &tokens_stats[token_denom];

        assert_eq!(token_stats.pending_stake, Uint128::zero());
        assert_eq!(token_stats.staked_shares_amount, Uint128::zero());
        assert_eq!(token_stats.pending_shares_unstake, Uint128::zero());

        let user_tokens_stats = get_user_tokens_stats(
            ctx.deps.as_ref(),
            ctx.env.clone(),
            ctx.user.clone(),
            &ctx.tokens,
        );

        let user_stats = &user_tokens_stats[token_denom];
        assert_eq!(user_stats.pending_stake, Uint128::zero());
        assert_eq!(user_stats.staked_shares_amount, Uint128::zero());
        assert_eq!(user_stats.pending_shares_unstake, Uint128::zero());

        // todo: check LP tokens are burned
    }

    #[test]
    fn test_unstake_response_fail() {
        let mut ctx = instantiate_contract();
        let (token_denom, token_config) = ctx.tokens.first().clone().unwrap();
        let stake_amount = Uint128::from(1000_u32);
        let shares_amount = stake_amount + Uint128::one();
        let return_amount = shares_amount + Uint128::one();

        // init stake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(
                &ctx.user.clone(),
                &vec![Coin {
                    amount: stake_amount,
                    denom: token_denom.clone(),
                }],
            ),
            ExecuteMsg::Stake,
        )
        .unwrap();

        // response for stake action
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.axelar.clone(), &vec![]),
            ExecuteMsg::HandleStakeResponse {
                account: ctx.user.clone(),
                token_evm: token_config.evm_address.clone(),
                token_amount: stake_amount,
                shares_amount,
                status: StakeStatus::Successful,
            },
        )
        .unwrap();

        // init unstake
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.user.clone(), &vec![]),
            ExecuteMsg::Unstake {
                token_denom: token_denom.clone(),
            },
        )
        .unwrap();

        // response for unstake action
        execute(
            ctx.deps.as_mut(),
            ctx.env.clone(),
            message_info(&ctx.axelar.clone(), &vec![]),
            ExecuteMsg::HandleUnstakeResponse {
                account: ctx.user.clone(),
                token_evm: token_config.evm_address.clone(),
                token_amount: return_amount,
                shares_amount,
                status: StakeStatus::Fail,
            },
        )
        .unwrap();

        // check stats
        let tokens_stats = get_tokens_stats(ctx.deps.as_ref(), ctx.env.clone());
        let token_stats = &tokens_stats[token_denom];

        assert_eq!(token_stats.pending_stake, Uint128::zero());
        assert_eq!(token_stats.staked_shares_amount, shares_amount);
        assert_eq!(token_stats.pending_shares_unstake, Uint128::zero());

        let user_tokens_stats = get_user_tokens_stats(
            ctx.deps.as_ref(),
            ctx.env.clone(),
            ctx.user.clone(),
            &ctx.tokens,
        );

        let user_stats = &user_tokens_stats[token_denom];
        assert_eq!(user_stats.pending_stake, Uint128::zero());
        assert_eq!(user_stats.staked_shares_amount, shares_amount);
        assert_eq!(user_stats.pending_shares_unstake, Uint128::zero());

        // todo: check LP tokens are burned
    }

    /// utility functions
    fn instantiate_contract() -> TestContext {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let admin = Addr::unchecked("admin_acc");
        let user = Addr::unchecked("user_acc");
        let axelar = Addr::unchecked("axelar_acc");
        let tokens = create_tokens_config();

        instantiate(
            deps.as_mut(),
            env.clone(),
            message_info(&admin, &[]),
            InstantiateMsg {
                tokens: tokens.clone(),
                axelar: axelar.clone(),
            },
        )
        .unwrap();

        TestContext {
            deps,
            env,
            admin,
            user,
            axelar,
            tokens,
        }
    }

    fn create_tokens_config() -> Vec<(TokenDenom, TokenConfig)> {
        vec![
            (
                "token_1".into(),
                TokenConfig {
                    is_stake_enabled: true,
                    is_unstake_enabled: false,
                    symbol: "TOKEN1".to_string(),
                    lp_token_denom: "TOKEN1_LP".to_string(),
                    evm_address: "0x0000000000000000000000000000000000000001".to_string(),
                    evm_yield_contract: "0x0000000000000000000000000000000000000011".to_string(),
                },
            ),
            (
                "token_2".into(),
                TokenConfig {
                    is_stake_enabled: true,
                    is_unstake_enabled: false,
                    symbol: "TOKEN2".to_string(),
                    lp_token_denom: "TOKEN2_LP".to_string(),
                    evm_address: "0x0000000000000000000000000000000000000002".to_string(),
                    evm_yield_contract: "0x0000000000000000000000000000000000000022".to_string(),
                },
            ),
        ]
    }

    fn get_tokens_stats(deps: Deps, env: Env) -> HashMap<TokenDenom, TokenStats> {
        let tokens_stats: GetTokensStatsResponse =
            from_json(query(deps, env, QueryMsg::TokensStats).unwrap()).unwrap();

        let stats: HashMap<_, _> = tokens_stats.stats.into_iter().collect();
        stats
    }

    fn get_token_stats(deps: Deps, env: Env, token_denom: &TokenDenom) -> TokenStats {
        let token_stats = get_tokens_stats(deps, env);

        token_stats[token_denom].clone()
    }

    fn get_user_tokens_stats(
        deps: Deps,
        env: Env,
        account: Addr,
        tokens: &Vec<(TokenDenom, TokenConfig)>,
    ) -> HashMap<TokenDenom, TokenStats> {
        let mut tokens_stats: HashMap<TokenDenom, TokenStats> = HashMap::new();

        for (token_denom, _) in tokens {
            let token_stats: GetUserStatsResponse = from_json(
                query(
                    deps,
                    env.clone(),
                    QueryMsg::UserStats {
                        account: account.clone(),
                        token_denom: token_denom.clone(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
            tokens_stats.insert(token_denom.clone(), token_stats.stats);
        }
        tokens_stats
    }

    fn get_user_token_stats(
        deps: Deps,
        env: Env,
        account: Addr,
        token_denom: TokenDenom,
    ) -> TokenStats {
        let token_stats: GetUserStatsResponse = from_json(
            query(
                deps,
                env.clone(),
                QueryMsg::UserStats {
                    account,
                    token_denom,
                },
            )
            .unwrap(),
        )
        .unwrap();

        token_stats.stats
    }
}
