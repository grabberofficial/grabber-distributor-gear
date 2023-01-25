#![no_std]

mod distributor;
mod utils;
pub mod io;

use gstd::{exec, msg, prelude::*};
use distributor::Distributor;
use crate::io::*;

static mut DISTRIBUTOR: Option<Distributor> = None;

gstd::metadata! {
    title: "Grabber",
    init: 
        input: InitDistributor,
    handle:
        input: DistributorAction,
        output: DistributorEvent,
    state:
        input: DistributorState,
        output: DistributorReply,
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let distributor = Distributor {
        admin: exec::origin(),
        ..Distributor::default()
    };

    DISTRIBUTOR = Some(distributor);
}

#[no_mangle]
extern "C" fn handle() {
    let action: DistributorAction = msg::load().expect("Could not load DistributorAction");
    let distributor: &mut Distributor = unsafe { DISTRIBUTOR.get_or_insert(Default::default()) };
    
    match action {
        DistributorAction::Register => {
            distributor.register();
        },
        DistributorAction::RegisterMultiple(addresses) => {
            distributor.register_multiple(addresses);
        },
        DistributorAction::SetRegistrationFee(amount) => {
            distributor.set_registration_fee(amount);
        },
        DistributorAction::SetRegistrationRound { start_datetime, end_datetime } => {
            distributor.set_registration_time(start_datetime, end_datetime);
        },
        DistributorAction::SetDistributionRound {
             start_datetime, end_datetime 
            } => {
            distributor.set_distribution_time(start_datetime, end_datetime);
        },
        DistributorAction::SetDistributionParameters {
            token,
            owner, 
            amount_of_tokens_to_distribute, 
            vesting_precision 
        } => {
            distributor.set_distribution_parameters(
                token, 
                owner, 
                amount_of_tokens_to_distribute, 
                vesting_precision);
        },
        DistributorAction::SetVestingParameters { 
            portions_unlock_times, 
            percents_per_portions 
        } => {
            distributor.set_vesting_parameters(
                portions_unlock_times,
                percents_per_portions
            );
        },
        DistributorAction::Participate => {
            distributor.participate();
        },
        DistributorAction::Withdraw => {
            distributor.withdraw();
        },
        DistributorAction::SetAllocationSize { user, amount } => {
            distributor.set_allocation_size(user, amount);
        },
        DistributorAction::DepositTokens => {
            distributor.deposit_tokens();
        },
        DistributorAction::WithdrawLeftover => {
            distributor.withdraw_leftover();
        },
        DistributorAction::WithdrawFee => {
            distributor.withdraw_fee();
        },
        DistributorAction::SetVesingEndDate(end_date) => {
            distributor.set_vesting_enddate(end_date);
        }
    }
}

#[no_mangle]
extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: DistributorState = msg::load().expect("Failed to decode input argument");
    let distributor: &mut Distributor = unsafe { DISTRIBUTOR.get_or_insert(Default::default()) };

    let encoded = match query {
        DistributorState::GetRegisteredUsers 
            => DistributorReply::RegisteredUsers(distributor.get_registered_users()),
        DistributorState::GetParticipatedUsers 
            => DistributorReply::ParticipatedUsers(distributor.get_participated_users()),
        DistributorState::GetClaimedUsers 
            => DistributorReply::ClaimedUsers(distributor.get_claimed_users()),
        DistributorState::GetRegistrationRound 
            => DistributorReply::RegistrationRoundDates { 
                start_datetime: distributor.registration.start_datetime, 
                end_datetime: distributor.registration.end_datetime 
            },
        DistributorState::GetDistributionRound 
            => DistributorReply::DistributionRoundDates { 
                start_datetime: distributor.distribution.start_datetime, 
                end_datetime: distributor.distribution.end_datetime
            },
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}
