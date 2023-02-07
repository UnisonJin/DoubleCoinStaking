use crate::contract::{compute_reward, compute_staker_reward};
use crate::msg::{
    ConfigResponse, QueryMsg, StakerInfoResponse, StakersListResponse, StateResponse,
    UnbondingInfoResponse,
};
use crate::state::{
    staker_info_key, staker_info_storage, unbonding_info_storage, user_earned_info_key,
    user_earned_info_storage, UnbondingInfo, CONFIG, STATE,
};
use cosmwasm_std::{entry_point, to_binary, Binary, Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { block_time } => to_binary(&query_state(deps, block_time)?),
        QueryMsg::StakerInfo { staker } => to_binary(&query_staker_info(deps, env, staker)?),
        QueryMsg::AllStakers { start_after, limit } => {
            to_binary(&query_all_stakers(deps, start_after, limit)?)
        }
        QueryMsg::UnbondingInfo {
            staker,
            start_after,
            limit,
        } => to_binary(&query_unbonding_info(
            deps,
            env,
            staker,
            start_after,
            limit,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        lp_token_contract: config.lp_token_contract,
        reward_token1: config.reward_token1,
        reward_token2: config.reward_token2,
        distribution_schedule_token1: config.distribution_schedule_token1,
        distribution_schedule_token2: config.distribution_schedule_token2,
        admin: config.admin,
        lock_duration: config.lock_duration,
    })
}

pub fn query_state(deps: Deps, block_time: Option<u64>) -> StdResult<StateResponse> {
    let mut state = STATE.load(deps.storage)?;
    if let Some(block_time) = block_time {
        let config = CONFIG.load(deps.storage)?;
        compute_reward(&config, &mut state, block_time);
    }

    Ok(StateResponse {
        last_distributed: state.last_distributed,
        total_bond_amount: state.total_bond_amount,
        global_reward_index_token1: state.global_reward_index_token1,
        global_reward_index_token2: state.global_reward_index_token2,
    })
}

pub fn query_staker_info(deps: Deps, env: Env, staker: String) -> StdResult<StakerInfoResponse> {
    let block_time = Some(env.block.time.seconds());
    let staker_info_key = staker_info_key(&staker);
    match staker_info_storage().may_load(deps.storage, staker_info_key)? {
        Some(some_staker_info) => {
            let mut staker_info = some_staker_info;
            if let Some(block_time) = block_time {
                let config = CONFIG.load(deps.storage)?;
                let mut state = STATE.load(deps.storage)?;

                compute_reward(&config, &mut state, block_time);
                compute_staker_reward(&state, &mut staker_info)?;
            }

            let total_earned_token1: Uint128;
            let total_earned_token2: Uint128;

            let user_earned_info_key = user_earned_info_key(&staker);
            match user_earned_info_storage().may_load(deps.storage, user_earned_info_key)? {
                Some(user_earned_info) => {
                    total_earned_token1 = user_earned_info.total_earned_token1;
                    total_earned_token2 = user_earned_info.total_earned_token2;
                }
                None => {
                    total_earned_token1 = Uint128::zero();
                    total_earned_token2 = Uint128::zero();
                }
            }

            Ok(StakerInfoResponse {
                staker,
                reward_index_token1: staker_info.reward_index_token1,
                reward_index_token2: staker_info.reward_index_token2,
                bond_amount: staker_info.bond_amount,
                pending_reward_token1: staker_info.pending_reward_token1,
                pending_reward_token2: staker_info.pending_reward_token2,
                total_earned_token1,
                total_earned_token2,
            })
        }
        None => Ok(StakerInfoResponse {
            staker,
            reward_index_token1: Decimal::zero(),
            reward_index_token2: Decimal::zero(),
            bond_amount: Uint128::zero(),
            pending_reward_token1: Uint128::zero(),
            pending_reward_token2: Uint128::zero(),
            total_earned_token1: Uint128::zero(),
            total_earned_token2: Uint128::zero(),
        }),
    }
}

pub fn query_all_stakers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<StakersListResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let stakers_list = staker_info_storage()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(StakersListResponse { stakers_list })
}

pub fn query_unbonding_info(
    deps: Deps,
    env: Env,
    staker: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<UnbondingInfoResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let unbonding_info = unbonding_info_storage()
        .idx
        .address
        .prefix(staker.clone())
        .range(
            deps.storage,
            Some(Bound::exclusive((staker, start_after.unwrap_or_default()))),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    let crr_time = env.block.time.seconds();

    Ok(UnbondingInfoResponse {
        unbonding_info,
        crr_time,
    })
}

pub fn query_all_unbonding_info(
    deps: Deps,
    _env: Env,
    staker: String,
) -> StdResult<Vec<UnbondingInfo>> {
    let unbonding_info = unbonding_info_storage()
        .idx
        .address
        .prefix(staker.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(unbonding_info)
}
