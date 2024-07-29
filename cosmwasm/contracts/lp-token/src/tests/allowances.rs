use crate::allowances::query_allowance;
use crate::contract::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::query::{query_balance, query_token_info};
use crate::ContractError;
use cosmwasm_std::testing::{message_info, mock_dependencies_with_balance, mock_env};
use cosmwasm_std::{
    attr, coins, Addr, Binary, CosmosMsg, Deps, DepsMut, StdError, SubMsg, Timestamp, Uint128,
    WasmMsg,
};
use cw20::{AllowanceResponse, Cw20Coin, Cw20ReceiveMsg, TokenInfoResponse};
use cw_utils::Expiration;

fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
    query_balance(deps, address.into()).unwrap().balance
}

// this will set up the instantiation for other tests
fn do_instantiate<T: Into<String>>(
    mut deps: DepsMut,
    addr: T,
    amount: Uint128,
) -> TokenInfoResponse {
    let instantiate_msg = InstantiateMsg {
        name: "Auto Gen".to_string(),
        symbol: "AUTO".to_string(),
        decimals: 3,
        initial_balances: vec![Cw20Coin {
            address: addr.into(),
            amount,
        }],
        mint: None,
        marketing: None,
    };
    let info = message_info(&Addr::unchecked("creator"), &[]);
    let env = mock_env();
    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    query_token_info(deps.as_ref()).unwrap()
}

#[test]
fn increase_decrease_allowances() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let owner = deps.api.addr_make("addr0001").to_string();
    let spender = deps.api.addr_make("addr0002").to_string();
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let env = mock_env();
    do_instantiate(deps.as_mut(), owner.clone(), Uint128::new(12340000));

    // no allowance to start
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    assert_eq!(allowance, AllowanceResponse::default());

    // set allowance with height expiration
    let allow1 = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: allow1,
        expires: Some(expires),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // ensure it looks good
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    assert_eq!(
        allowance,
        AllowanceResponse {
            allowance: allow1,
            expires
        }
    );

    // decrease it a bit with no expire set - stays the same
    let lower = Uint128::new(4444);
    let allow2 = allow1.checked_sub(lower).unwrap();
    let msg = ExecuteMsg::DecreaseAllowance {
        spender: spender.clone(),
        amount: lower,
        expires: None,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    assert_eq!(
        allowance,
        AllowanceResponse {
            allowance: allow2,
            expires
        }
    );

    // increase it some more and override the expires
    let raise = Uint128::new(87654);
    let allow3 = allow2 + raise;
    let new_expire = Expiration::AtTime(Timestamp::from_seconds(8888888888));
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: raise,
        expires: Some(new_expire),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    assert_eq!(
        allowance,
        AllowanceResponse {
            allowance: allow3,
            expires: new_expire
        }
    );

    // decrease it below 0
    let msg = ExecuteMsg::DecreaseAllowance {
        spender: spender.clone(),
        amount: Uint128::new(99988647623876347),
        expires: None,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();
    let allowance = query_allowance(deps.as_ref(), owner, spender).unwrap();
    assert_eq!(allowance, AllowanceResponse::default());
}

#[test]
fn allowances_independent() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let owner = deps.api.addr_make("addr0001").to_string();
    let spender = deps.api.addr_make("addr0002").to_string();
    let spender2 = deps.api.addr_make("addr0003").to_string();
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let env = mock_env();
    do_instantiate(deps.as_mut(), &owner, Uint128::new(12340000));

    // no allowance to start
    assert_eq!(
        query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap(),
        AllowanceResponse::default()
    );
    assert_eq!(
        query_allowance(deps.as_ref(), owner.clone(), spender2.clone()).unwrap(),
        AllowanceResponse::default()
    );
    assert_eq!(
        query_allowance(deps.as_ref(), spender.clone(), spender2.clone()).unwrap(),
        AllowanceResponse::default()
    );

    // set allowance with height expiration
    let allow1 = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: allow1,
        expires: Some(expires),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // set other allowance with no expiration
    let allow2 = Uint128::new(87654);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender2.clone(),
        amount: allow2,
        expires: None,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // check they are proper
    let expect_one = AllowanceResponse {
        allowance: allow1,
        expires,
    };
    let expect_two = AllowanceResponse {
        allowance: allow2,
        expires: Expiration::Never {},
    };
    assert_eq!(
        query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap(),
        expect_one
    );
    assert_eq!(
        query_allowance(deps.as_ref(), owner.clone(), spender2.clone()).unwrap(),
        expect_two
    );
    assert_eq!(
        query_allowance(deps.as_ref(), spender.clone(), spender2.clone()).unwrap(),
        AllowanceResponse::default()
    );

    // also allow spender -> spender2 with no interference
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let env = mock_env();
    let allow3 = Uint128::new(1821);
    let expires3 = Expiration::AtTime(Timestamp::from_seconds(3767626296));
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender2.clone(),
        amount: allow3,
        expires: Some(expires3),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();
    let expect_three = AllowanceResponse {
        allowance: allow3,
        expires: expires3,
    };
    assert_eq!(
        query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap(),
        expect_one
    );
    assert_eq!(
        query_allowance(deps.as_ref(), owner, spender2.clone()).unwrap(),
        expect_two
    );
    assert_eq!(
        query_allowance(deps.as_ref(), spender, spender2).unwrap(),
        expect_three
    );
}

#[test]
fn no_self_allowance() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let owner = deps.api.addr_make("addr0001").to_string();
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let env = mock_env();
    do_instantiate(deps.as_mut(), &owner, Uint128::new(12340000));

    // self-allowance
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: owner.clone(),
        amount: Uint128::new(7777),
        expires: None,
    };
    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert_eq!(err, ContractError::CannotSetOwnAccount {});

    // decrease self-allowance
    let msg = ExecuteMsg::DecreaseAllowance {
        spender: owner,
        amount: Uint128::new(7777),
        expires: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::CannotSetOwnAccount {});
}

#[test]
fn transfer_from_respects_limits() {
    let mut deps = mock_dependencies_with_balance(&[]);
    let owner = deps.api.addr_make("addr0001").to_string();
    let spender = deps.api.addr_make("addr0002").to_string();
    let rcpt = deps.api.addr_make("addr0003").to_string();

    let start = Uint128::new(999999);
    do_instantiate(deps.as_mut(), &owner, start);

    // provide an allowance
    let allow1 = Uint128::new(77777);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: allow1,
        expires: None,
    };
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let env = mock_env();
    execute(deps.as_mut(), env, info, msg).unwrap();

    // valid transfer of part of the allowance
    let transfer = Uint128::new(44444);
    let msg = ExecuteMsg::TransferFrom {
        owner: owner.clone(),
        recipient: rcpt.clone(),
        amount: transfer,
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.attributes[0], attr("action", "transfer_from"));

    // make sure money arrived
    assert_eq!(
        get_balance(deps.as_ref(), owner.clone()),
        start.checked_sub(transfer).unwrap()
    );
    assert_eq!(get_balance(deps.as_ref(), rcpt.clone()), transfer);

    // ensure it looks good
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    let expect = AllowanceResponse {
        allowance: allow1.checked_sub(transfer).unwrap(),
        expires: Expiration::Never {},
    };
    assert_eq!(expect, allowance);

    // cannot send more than the allowance
    let msg = ExecuteMsg::TransferFrom {
        owner: owner.clone(),
        recipient: rcpt.clone(),
        amount: Uint128::new(33443),
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to expire in the next block
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let mut env = mock_env();
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: Uint128::new(1000),
        expires: Some(Expiration::AtHeight(env.block.height + 1)),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    env.block.height += 1;

    // we should now get the expiration error
    let msg = ExecuteMsg::TransferFrom {
        owner,
        recipient: rcpt,
        amount: Uint128::new(33443),
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Expired {});
}

#[test]
fn burn_from_respects_limits() {
    let mut deps = mock_dependencies_with_balance(&[]);
    let owner = deps.api.addr_make("addr0001").to_string();
    let spender = deps.api.addr_make("addr0002").to_string();

    let start = Uint128::new(999999);
    do_instantiate(deps.as_mut(), &owner, start);

    // provide an allowance
    let allow1 = Uint128::new(77777);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: allow1,
        expires: None,
    };
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let env = mock_env();
    execute(deps.as_mut(), env, info, msg).unwrap();

    // valid burn of part of the allowance
    let transfer = Uint128::new(44444);
    let msg = ExecuteMsg::BurnFrom {
        owner: owner.clone(),
        amount: transfer,
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.attributes[0], attr("action", "burn_from"));

    // make sure money burnt
    assert_eq!(
        get_balance(deps.as_ref(), owner.clone()),
        start.checked_sub(transfer).unwrap()
    );

    // ensure it looks good
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    let expect = AllowanceResponse {
        allowance: allow1.checked_sub(transfer).unwrap(),
        expires: Expiration::Never {},
    };
    assert_eq!(expect, allowance);

    // cannot burn more than the allowance
    let msg = ExecuteMsg::BurnFrom {
        owner: owner.clone(),
        amount: Uint128::new(33443),
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to expire in the next block
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let mut env = mock_env();
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: Uint128::new(1000),
        expires: Some(Expiration::AtHeight(env.block.height + 1)),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // increase block height, so the limit is expired now
    env.block.height += 1;

    // we should now get the expiration error
    let msg = ExecuteMsg::BurnFrom {
        owner,
        amount: Uint128::new(33443),
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Expired {});
}

#[test]
fn send_from_respects_limits() {
    let mut deps = mock_dependencies_with_balance(&[]);
    let owner = deps.api.addr_make("addr0001").to_string();
    let spender = deps.api.addr_make("addr0002").to_string();
    let contract = deps.api.addr_make("addr0003").to_string();
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());

    let start = Uint128::new(999999);
    do_instantiate(deps.as_mut(), &owner, start);

    // provide an allowance
    let allow1 = Uint128::new(77777);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: allow1,
        expires: None,
    };
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let env = mock_env();
    execute(deps.as_mut(), env, info, msg).unwrap();

    // valid send of part of the allowance
    let transfer = Uint128::new(44444);
    let msg = ExecuteMsg::SendFrom {
        owner: owner.clone(),
        amount: transfer,
        contract: contract.clone(),
        msg: send_msg.clone(),
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.attributes[0], attr("action", "send_from"));
    assert_eq!(1, res.messages.len());

    // we record this as sent by the one who requested, not the one who was paying
    let binary_msg = Cw20ReceiveMsg {
        sender: spender.clone(),
        amount: transfer,
        msg: send_msg.clone(),
    }
    .into_json_binary()
    .unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract.clone(),
            msg: binary_msg,
            funds: vec![],
        }))
    );

    // make sure money sent
    assert_eq!(
        get_balance(deps.as_ref(), owner.clone()),
        start.checked_sub(transfer).unwrap()
    );
    assert_eq!(get_balance(deps.as_ref(), contract.clone()), transfer);

    // ensure it looks good
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    let expect = AllowanceResponse {
        allowance: allow1.checked_sub(transfer).unwrap(),
        expires: Expiration::Never {},
    };
    assert_eq!(expect, allowance);

    // cannot send more than the allowance
    let msg = ExecuteMsg::SendFrom {
        owner: owner.clone(),
        amount: Uint128::new(33443),
        contract: contract.clone(),
        msg: send_msg.clone(),
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to the next block
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let mut env = mock_env();
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: Uint128::new(1000),
        expires: Some(Expiration::AtHeight(env.block.height + 1)),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // increase block height, so the limit is expired now
    env.block.height += 1;

    // we should now get the expiration error
    let msg = ExecuteMsg::SendFrom {
        owner,
        amount: Uint128::new(33443),
        contract,
        msg: send_msg,
    };
    let info = message_info(&Addr::unchecked(spender.clone()), &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Expired {});
}

#[test]
fn no_past_expiration() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let owner = deps.api.addr_make("addr0001").to_string();
    let spender = deps.api.addr_make("addr0002").to_string();
    let info = message_info(&Addr::unchecked(owner.clone()), &[]);
    let env = mock_env();
    do_instantiate(deps.as_mut(), owner.clone(), Uint128::new(12340000));

    // set allowance with height expiration at current block height
    let expires = Expiration::AtHeight(env.block.height);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: Uint128::new(7777),
        expires: Some(expires),
    };

    // ensure it is rejected
    assert_eq!(
        Err(ContractError::InvalidExpiration {}),
        execute(deps.as_mut(), env.clone(), info.clone(), msg)
    );

    // set allowance with time expiration in the past
    let expires = Expiration::AtTime(env.block.time.minus_seconds(1));
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: Uint128::new(7777),
        expires: Some(expires),
    };

    // ensure it is rejected
    assert_eq!(
        Err(ContractError::InvalidExpiration {}),
        execute(deps.as_mut(), env.clone(), info.clone(), msg)
    );

    // set allowance with height expiration at next block height
    let expires = Expiration::AtHeight(env.block.height + 1);
    let allow = Uint128::new(7777);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: allow,
        expires: Some(expires),
    };

    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // ensure it looks good
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    assert_eq!(
        allowance,
        AllowanceResponse {
            allowance: allow,
            expires
        }
    );

    // set allowance with time expiration in the future
    let expires = Expiration::AtTime(env.block.time.plus_seconds(10));
    let allow = Uint128::new(7777);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: allow,
        expires: Some(expires),
    };

    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // ensure it looks good
    let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
    assert_eq!(
        allowance,
        AllowanceResponse {
            allowance: allow + allow, // we increased twice
            expires
        }
    );

    // decrease with height expiration at current block height
    let expires = Expiration::AtHeight(env.block.height);
    let allow = Uint128::new(7777);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: spender.clone(),
        amount: allow,
        expires: Some(expires),
    };

    // ensure it is rejected
    assert_eq!(
        Err(ContractError::InvalidExpiration {}),
        execute(deps.as_mut(), env.clone(), info.clone(), msg)
    );

    // decrease with height expiration at next block height
    let expires = Expiration::AtHeight(env.block.height + 1);
    let allow = Uint128::new(7777);
    let msg = ExecuteMsg::DecreaseAllowance {
        spender: spender.clone(),
        amount: allow,
        expires: Some(expires),
    };

    execute(deps.as_mut(), env, info, msg).unwrap();

    // ensure it looks good
    let allowance = query_allowance(deps.as_ref(), owner, spender).unwrap();
    assert_eq!(
        allowance,
        AllowanceResponse {
            allowance: allow,
            expires
        }
    );
}
