use gstd::{prelude::*, ActorId};

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitDistributor {}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum DistributorAction {
    Register,
    RegisterMultiple(Vec<ActorId>),

    Participate,
    Withdraw,
    
    SetRegistrationFee(u128),
    SetRegistrationRound {
        start_datetime: u64,
        end_datetime: u64
    },
    SetDistributionRound {
        start_datetime: u64,
        end_datetime: u64
    },
    SetAllocationSize{
        user: ActorId,
        amount: u128 
    },
    SetDistributionParameters {
        token: ActorId,
        owner: ActorId,
        amount_of_tokens_to_distribute: u128,
        vesting_precision: u128
    },
    SetVestingParameters {
        portions_unlock_times: Vec<u64>,
        percents_per_portions: Vec<u128>
    },
    SetVesingEndDate(u64),

    WithdrawLeftover,
    WithdrawFee,
    DepositTokens
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum DistributorEvent {
    Registered {
        who: ActorId,
        when: u64
    },
    Participated {
        who: ActorId,
        when: u64
    },
    
    RegistrationFeeSet(u64),
    RegistrationRoundSet(u64),
    DistributionRoundSet(u64),

    DistributionParametersSet(u64),
    VestingParametersSet(u64),

    Withdrawn(u64),
    LeftoverWithdrawn(u64),
    FeeWithdrawn(u128),

    TokensDeposited(u64)
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum DistributorState {
    GetRegisteredUsers,
    GetParticipatedUsers,
    GetClaimedUsers,

    GetRegistrationRound,
    GetDistributionRound
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum DistributorReply {
    RegisteredUsers(Vec<ActorId>),
    ParticipatedUsers(Vec<ActorId>),
    ClaimedUsers(Vec<ActorId>),

    RegistrationRoundDates {
        start_datetime: u64,
        end_datetime: u64
    },
    DistributionRoundDates {
        start_datetime: u64,
        end_datetime: u64
    }
}
