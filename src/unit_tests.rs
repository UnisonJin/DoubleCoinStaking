#[cfg(test)]
use crate::contract::{execute, instantiate};
use crate::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use crate::query::{query_all_unbonding_info, query_staker_info, query_unbonding_info};
use crate::state::Denom;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_binary, Addr, CosmosMsg, DepsMut, Env, Uint128, WasmMsg};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

fn setup_contract(deps: DepsMut, env: Env) {
    let instantiate_msg = InstantiateMsg {
        lp_token_contract: "lp_token_contract".to_string(),
        reward_token1: Denom::Cw20(Addr::unchecked("reward_token1")),
        reward_token2: Denom::Cw20(Addr::unchecked("reward_token1")),
        lock_duration: 3600,
        distribution_schedule_token1: vec![(
            env.block.time.seconds(),
            env.block.time.seconds() + 86400,
            Uint128::new(100000000),
        )],
        distribution_schedule_token2: vec![(
            env.block.time.seconds(),
            env.block.time.seconds() + 86400,
            Uint128::new(100000000),
        )],
    };
    let info = mock_info("owner", &[]);
    let res = instantiate(deps, mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_earned() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    setup_contract(deps.as_mut(), env.clone());

    let info = mock_info("lp_token_contract", &[]);
    let hook_msg = Cw20HookMsg::Bond {};
    let cw20_rcv_msg = Cw20ReceiveMsg {
        sender: "user1".to_string(),
        amount: Uint128::new(1000),
        msg: to_binary(&hook_msg).unwrap(),
    };
    let msg = ExecuteMsg::Receive(cw20_rcv_msg);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info("lp_token_contract", &[]);
    let hook_msg = Cw20HookMsg::Bond {};
    let cw20_rcv_msg = Cw20ReceiveMsg {
        sender: "user2".to_string(),
        amount: Uint128::new(500),
        msg: to_binary(&hook_msg).unwrap(),
    };
    let msg = ExecuteMsg::Receive(cw20_rcv_msg);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    env.block.time = env.block.time.plus_seconds(800);

    let staker1_info = query_staker_info(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();
    let staker2_info = query_staker_info(deps.as_ref(), env.clone(), "user2".to_string()).unwrap();

    println!("{:?}, {:?}", staker1_info, staker2_info);

    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Withdraw {};
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let staker1_info = query_staker_info(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();
    // let staker2_info = query_staker_info(deps.as_ref(), env.clone(), "user2".to_string()).unwrap();

    println!("{:?}", staker1_info);

    env.block.time = env.block.time.plus_seconds(300);
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Withdraw {};
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let staker1_info = query_staker_info(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();
    // let staker2_info = query_staker_info(deps.as_ref(), env.clone(), "user2".to_string()).unwrap();

    println!("{:?}", staker1_info);
}

#[test]
fn test_unbond() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    setup_contract(deps.as_mut(), env.clone());

    let info = mock_info("lp_token_contract", &[]);
    let hook_msg = Cw20HookMsg::Bond {};
    let cw20_rcv_msg = Cw20ReceiveMsg {
        sender: "user1".to_string(),
        amount: Uint128::new(1000),
        msg: to_binary(&hook_msg).unwrap(),
    };
    let msg = ExecuteMsg::Receive(cw20_rcv_msg);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info("lp_token_contract", &[]);
    let hook_msg = Cw20HookMsg::Bond {};
    let cw20_rcv_msg = Cw20ReceiveMsg {
        sender: "user2".to_string(),
        amount: Uint128::new(500),
        msg: to_binary(&hook_msg).unwrap(),
    };
    let msg = ExecuteMsg::Receive(cw20_rcv_msg);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(200),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    env.block.time = env.block.time.plus_seconds(500);
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(300),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let unbonding_info =
        query_unbonding_info(deps.as_ref(), env.clone(), "user1".to_string(), None, None).unwrap();
    println!("unbonding_info: {:?}", unbonding_info);

    let all_unbonding_info =
        query_all_unbonding_info(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();

    println!("all_unbonding_info: {:?}", all_unbonding_info);

    env.block.time = env.block.time.plus_seconds(3650);

    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Redeem {};
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "lp_token_contract".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "user1".to_string(),
                amount: Uint128::new(500)
            })
            .unwrap(),
            funds: vec![]
        })
    );

    let unbonding_info =
        query_unbonding_info(deps.as_ref(), env.clone(), "user1".to_string(), None, None).unwrap();
    println!("unbonding_info_after_redeem: {:?}", unbonding_info);
}

#[test]
fn test_withdraw() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    setup_contract(deps.as_mut(), env.clone());

    let info = mock_info("lp_token_contract", &[]);
    let hook_msg = Cw20HookMsg::Bond {};
    let cw20_rcv_msg = Cw20ReceiveMsg {
        sender: "user1".to_string(),
        amount: Uint128::new(1000),
        msg: to_binary(&hook_msg).unwrap(),
    };
    let msg = ExecuteMsg::Receive(cw20_rcv_msg);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    env.block.time = env.block.time.plus_seconds(800);

    let staker1_info = query_staker_info(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();
    println!("staker1_info,{:?}", staker1_info);

    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Withdraw {};
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 2);
}
