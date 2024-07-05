use crate::state::{
    TokenStats, CONTRACT_CONFIG_STATE, TOKENS_CONFIGS_STATE, TOKENS_STATS_STATE, USERS_STATS_STATE,
};
use crate::types::{StakeStatus, TokenConfig, TokenDenom};
use crate::ContractError;
use cosmwasm_std::{
    Addr, BankMsg, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128,
};

pub fn try_stake(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    for coin in &info.funds {
        let token_config = TOKENS_CONFIGS_STATE
            .may_load(deps.storage, coin.denom.clone())?
            .ok_or(ContractError::UnknownToken(coin.denom.clone()))?;

        // check is staking enabled
        if !token_config.is_stake_enabled {
            return Err(ContractError::StakeDisabled(token_config.symbol));
        }

        // todo: send tokens to Axelar

        // update user stats
        let mut user_stats = USERS_STATS_STATE
            .may_load(deps.storage, (info.sender.clone(), coin.denom.clone()))?
            .unwrap_or(TokenStats {
                pending_stake: Uint128::zero(),
                staked_shares_amount: Uint128::zero(),
                pending_shares_unstake: Uint128::zero(),
            });

        user_stats.pending_stake += coin.amount;
        USERS_STATS_STATE.save(
            deps.storage,
            (info.sender.clone(), coin.denom.clone()),
            &user_stats,
        )?;

        // update token stats
        let mut token_stats = TOKENS_STATS_STATE.load(deps.storage, coin.denom.clone())?;
        token_stats.pending_stake += coin.amount;
        TOKENS_STATS_STATE.save(deps.storage, coin.denom.clone(), &token_stats)?;
    }

    Ok(Response::default())
}

pub fn try_add_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: String,
    config: TokenConfig,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    if TOKENS_CONFIGS_STATE.has(deps.storage, token_denom.clone()) {
        return Err(ContractError::TokenAlreadyExist(token_denom));
    }

    TOKENS_CONFIGS_STATE.save(deps.storage, token_denom.clone(), &config)?;

    TOKENS_STATS_STATE.save(
        deps.storage,
        token_denom.clone(),
        &TokenStats {
            pending_stake: Uint128::default(),
            staked_shares_amount: Uint128::default(),
            pending_shares_unstake: Uint128::default(),
        },
    )?;

    Ok(Response::default())
}

pub fn try_update_token_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: String,
    config: TokenConfig,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_admin(deps.as_ref(), &info)?;

    if !TOKENS_CONFIGS_STATE.has(deps.storage, token_denom.clone()) {
        return Err(ContractError::UnknownToken(token_denom.clone()));
    }
    TOKENS_CONFIGS_STATE.save(deps.storage, token_denom, &config)?;

    Ok(Response::default())
}

pub fn try_handle_stake_response(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    account: Addr,
    token_evm: String,
    token_amount: Uint128,
    shares_amount: Uint128,
    status: StakeStatus,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_axelar(deps.as_ref(), &info)?;

    let tokens_configs: StdResult<Vec<(TokenDenom, TokenConfig)>> = TOKENS_CONFIGS_STATE
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let tokens_configs = tokens_configs?;

    let (token_denom, _) = tokens_configs
        .iter()
        .find(|(_, config)| config.evm_address.to_lowercase() == token_evm.to_lowercase())
        .ok_or(ContractError::UnknownToken(token_evm))?;

    let mut token_stats = TOKENS_STATS_STATE.load(deps.storage, token_denom.clone())?;
    let mut user_stats =
        USERS_STATS_STATE.load(deps.storage, (account.clone(), token_denom.clone()))?;

    // todo: any checks?
    // if user_stats.pending_stake < amount {
    //     return Err(ContractError::CustomError(
    //         "Pending staking amount < amount from message".to_string(),
    //     ));
    // }

    if status == StakeStatus::Successful {
        if info.funds.len() > 0 {
            return Err(ContractError::CustomError(
                "Successful stake response has coins in message".to_string(),
            ));
        }
        // update token and user stats
        token_stats.pending_stake -= token_amount.clone();
        user_stats.pending_stake -= token_amount.clone();

        token_stats.staked_shares_amount += shares_amount.clone();
        user_stats.staked_shares_amount += shares_amount;

        TOKENS_STATS_STATE.save(deps.storage, token_denom.clone(), &token_stats)?;
        USERS_STATS_STATE.save(deps.storage, (account, token_denom.clone()), &user_stats)?;

        return Ok(Response::default());
    } else {
        if info.funds.len() > 1 {
            return Err(ContractError::CustomError(
                "Failed stake response has too much coins in message".to_string(),
            ));
        }
        let coin = info.funds.first().unwrap();
        if coin.denom != *token_denom {
            return Err(ContractError::InvalidToken {
                expected: token_denom.clone(),
                actual: coin.denom.clone(),
            });
        }

        // update token and user stats
        token_stats.pending_stake -= token_amount.clone();
        user_stats.pending_stake -= token_amount;

        TOKENS_STATS_STATE.save(deps.storage, token_denom.clone(), &token_stats)?;
        USERS_STATS_STATE.save(
            deps.storage,
            (account.clone(), token_denom.clone()),
            &user_stats,
        )?;

        // todo: add attributes? events?
        return Ok(Response::new().add_message(BankMsg::Send {
            to_address: account.to_string(),
            amount: info.funds,
        }));
    }
}

pub fn try_handle_unstake_response(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    account: Addr,
    token_evm: String,
    _token_amount: Uint128,
    shares_amount: Uint128,
    status: StakeStatus,
) -> Result<Response, ContractError> {
    assert_msg_sender_is_axelar(deps.as_ref(), &info)?;

    let tokens_configs: StdResult<Vec<(TokenDenom, TokenConfig)>> = TOKENS_CONFIGS_STATE
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let tokens_configs = tokens_configs?;

    let (token_denom, _) = tokens_configs
        .iter()
        .find(|(_, config)| config.evm_address.to_lowercase() == token_evm.to_lowercase())
        .ok_or(ContractError::UnknownToken(token_evm))?;

    let mut token_stats = TOKENS_STATS_STATE.load(deps.storage, token_denom.clone())?;
    let mut user_stats =
        USERS_STATS_STATE.load(deps.storage, (account.clone(), token_denom.clone()))?;

    if user_stats.pending_shares_unstake < shares_amount {
        return Err(ContractError::CustomError(
            "Pending unstake amount < amount from message".to_string(),
        ));
    }

    if status == StakeStatus::Successful {
        if info.funds.len() > 1 {
            return Err(ContractError::CustomError(
                "Successful unstake response has too much coins in message".to_string(),
            ));
        }

        let coin = info.funds.first().unwrap();
        if coin.denom != *token_denom {
            return Err(ContractError::InvalidToken {
                expected: token_denom.clone(),
                actual: coin.denom.clone(),
            });
        }

        // update token and user stats
        token_stats.pending_shares_unstake -= shares_amount.clone();
        user_stats.pending_shares_unstake -= shares_amount.clone();

        TOKENS_STATS_STATE.save(deps.storage, token_denom.clone(), &token_stats)?;
        USERS_STATS_STATE.save(
            deps.storage,
            (account.clone(), token_denom.clone()),
            &user_stats,
        )?;

        return Ok(Response::new().add_message(BankMsg::Send {
            to_address: account.to_string(),
            amount: vec![coin.clone()],
        }));
    } else {
        if info.funds.len() > 0 {
            return Err(ContractError::CustomError(
                "Failed unstake response has coins in message".to_string(),
            ));
        }

        // update token and user stats
        token_stats.pending_shares_unstake -= shares_amount.clone();
        user_stats.pending_shares_unstake -= shares_amount.clone();

        token_stats.staked_shares_amount += shares_amount.clone();
        user_stats.staked_shares_amount += shares_amount.clone();

        TOKENS_STATS_STATE.save(deps.storage, token_denom.clone(), &token_stats)?;
        USERS_STATS_STATE.save(
            deps.storage,
            (account.clone(), token_denom.clone()),
            &user_stats,
        )?;
        // todo: return LP tokens
    }

    Ok(Response::default())
}

pub fn try_unstake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_denom: TokenDenom,
) -> Result<Response, ContractError> {
    // if info.funds.len() != 1 {
    //     return Err(ContractError::CustomError(
    //         "Successful unstake response has too much coins in message".to_string(),
    //     ));
    // }
    //
    // let coin = info.funds.first().unwrap();
    //
    // let tokens_configs: StdResult<Vec<(TokenDenom, TokenConfig)>> = TOKENS_CONFIGS_STATE
    //     .range(deps.storage, None, None, Order::Ascending)
    //     .collect();
    //
    // let tokens_configs = tokens_configs?;
    //
    // let (token_denom, _) = tokens_configs
    //     .iter()
    //     .find(|(_, config)| config.lp_token_denom == coin.denom)
    //     .ok_or(ContractError::UnknownLpToken(coin.denom.clone()))?;
    //
    let mut user_stats =
        USERS_STATS_STATE.load(deps.storage, (info.sender.clone(), token_denom.clone()))?;

    if user_stats.staked_shares_amount.is_zero() {
        return Err(ContractError::NothingToUnstake);
    }

    let mut token_stats = TOKENS_STATS_STATE.load(deps.storage, token_denom.clone())?;

    // update user and token stats
    user_stats.pending_shares_unstake += user_stats.staked_shares_amount;
    token_stats.pending_shares_unstake += user_stats.staked_shares_amount;
    token_stats.staked_shares_amount -= user_stats.staked_shares_amount;
    user_stats.staked_shares_amount = Uint128::zero();

    USERS_STATS_STATE.save(
        deps.storage,
        (info.sender.clone(), token_denom.clone()),
        &user_stats,
    )?;

    TOKENS_STATS_STATE.save(deps.storage, token_denom.clone(), &token_stats)?;

    // todo: send message to Axelar

    Ok(Response::default())
}

fn assert_msg_sender_is_admin(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let contract_config = CONTRACT_CONFIG_STATE.load(deps.storage)?;
    if contract_config.owner != info.sender {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

fn assert_msg_sender_is_axelar(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let contract_config = CONTRACT_CONFIG_STATE.load(deps.storage)?;
    if contract_config.axelar != info.sender {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}
