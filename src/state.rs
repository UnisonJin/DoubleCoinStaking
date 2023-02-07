use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config_config");
pub const STATE: Item<State> = Item::new("config_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Denom {
    Native(String),
    Cw20(Addr),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub lp_token_contract: String,
    pub reward_token1: Denom,
    pub reward_token2: Denom,
    pub distribution_schedule_token1: Vec<(u64, u64, Uint128)>,
    pub distribution_schedule_token2: Vec<(u64, u64, Uint128)>,
    pub admin: String,
    pub lock_duration: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub last_distributed: u64,
    pub total_bond_amount: Uint128,
    pub global_reward_index_token1: Decimal,
    pub global_reward_index_token2: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub address: String,
    pub bond_amount: Uint128,
    pub reward_index_token1: Decimal,
    pub reward_index_token2: Decimal,
    pub pending_reward_token1: Uint128,
    pub pending_reward_token2: Uint128,
}

pub type StakerInfoKey<'a> = String;

pub fn staker_info_key<'a>(address: &'a String) -> StakerInfoKey<'a> {
    address.clone()
}

pub struct StakerInfoIndicies<'a> {
    pub address: MultiIndex<'a, String, StakerInfo, StakerInfoKey<'a>>,
}

impl<'a> IndexList<StakerInfo> for StakerInfoIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<StakerInfo>> + '_> {
        let v: Vec<&dyn Index<StakerInfo>> = vec![&self.address];
        Box::new(v.into_iter())
    }
}

pub fn staker_info_storage<'a>(
) -> IndexedMap<'a, StakerInfoKey<'a>, StakerInfo, StakerInfoIndicies<'a>> {
    let indexes = StakerInfoIndicies {
        address: MultiIndex::new(
            |d: &StakerInfo| d.address.clone(),
            "staker_info",
            "staker_info__collection",
        ),
    };
    IndexedMap::new("staker_info", indexes)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserEarnedInfo {
    pub address: String,
    pub total_earned_token1: Uint128,
    pub total_earned_token2: Uint128,
}

pub type UserEarnedInfoKey<'a> = String;

pub fn user_earned_info_key<'a>(address: &'a String) -> UserEarnedInfoKey<'a> {
    address.clone()
}

pub struct UserEarnedInfoIndicies<'a> {
    pub address: MultiIndex<'a, String, UserEarnedInfo, UserEarnedInfoKey<'a>>,
}

impl<'a> IndexList<UserEarnedInfo> for UserEarnedInfoIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<UserEarnedInfo>> + '_> {
        let v: Vec<&dyn Index<UserEarnedInfo>> = vec![&self.address];
        Box::new(v.into_iter())
    }
}

pub fn user_earned_info_storage<'a>(
) -> IndexedMap<'a, UserEarnedInfoKey<'a>, UserEarnedInfo, UserEarnedInfoIndicies<'a>> {
    let indexes = UserEarnedInfoIndicies {
        address: MultiIndex::new(
            |d: &UserEarnedInfo| d.address.clone(),
            "user_earned_info",
            "user_earned_info_address",
        ),
    };
    IndexedMap::new("user_earned_info", indexes)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondingInfo {
    pub address: String,
    pub time: u64,
    pub amount: Uint128,
}

pub type UnbondingInfoKey<'a> = (String, u64);

pub fn unbonding_info_key<'a>(address: &String, time: u64) -> UnbondingInfoKey {
    (address.clone(), time)
}

pub struct UnbondingInfoIndicies<'a> {
    pub address: MultiIndex<'a, String, UnbondingInfo, UnbondingInfoKey<'a>>,
}

impl<'a> IndexList<UnbondingInfo> for UnbondingInfoIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<UnbondingInfo>> + '_> {
        let v: Vec<&dyn Index<UnbondingInfo>> = vec![&self.address];
        Box::new(v.into_iter())
    }
}

pub fn unbonding_info_storage<'a>(
) -> IndexedMap<'a, UnbondingInfoKey<'a>, UnbondingInfo, UnbondingInfoIndicies<'a>> {
    let indexes = UnbondingInfoIndicies {
        address: MultiIndex::new(
            |d: &UnbondingInfo| d.address.clone(),
            "unbonding_info",
            "user_unbonding_info",
        ),
    };
    IndexedMap::new("unbonding_info", indexes)
}
