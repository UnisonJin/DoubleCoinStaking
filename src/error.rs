use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("This user did not take part in staking")]
    NotStaked {},

    #[error("There is nothing to redeem")]
    NothingToRedeem {},

    #[error("You used wrong token contract")]
    WrongContractError {},

    #[error("You should send the bond message when you call this function")]
    DataShouldBeGiven {},

    #[error("You should wait until the lock time is finished")]
    TimeRemainingForRedeem {},

    #[error("Cannot update; the new schedule must support all of the previous schedule")]
    NotIncludeAllDistributionSchedule {},

    #[error("new schedule removes already started distribution")]
    NewScheduleRemovePastDistribution {},

    #[error("new schedule adds an already started distribution")]
    NewScheduleAddPastDistribution {},

    #[error("Cannot unbond more than bond amount")]
    ExceedBondAmount {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },
}
