use crate::io::*;
use ft_io::*;
use gstd::{exec, msg, prelude::*, ActorId, Vec, BTreeMap, debug};

use crate::require;

#[derive(Debug, Default, Clone)]
pub struct RegistrationRound {
    pub start_datetime: u64,
    pub end_datetime: u64,
    pub users: BTreeMap<ActorId, u128>
}

#[derive(Debug, Default, Clone)]
pub struct DistributionRound {
    pub start_datetime: u64,
    pub end_datetime: u64,
    pub users: BTreeMap<ActorId, bool>
}

#[derive(Debug, Default, Clone)]
pub struct VestingParameters {
    pub end_datetime: u64,
    pub portions_unlock_times: Vec<u64>,
    pub percents_per_portions: Vec<u128>,
    pub precision: u128
}

#[derive(Debug, Default, Clone)]
pub struct DistributionParameters {
    pub token: ActorId,
    pub owner: ActorId,
    pub amount_of_tokens_to_distribute: u128
}

#[derive(Debug, Default)]
pub struct Distributor {
    pub admin: ActorId,

    pub registration: RegistrationRound,
    pub distribution: DistributionRound,

    pub vesting: VestingParameters,
    pub vesting_enddate: u64,
    pub distribution_parameters: DistributionParameters,
    pub total_tokens_distributed: u128,
    pub registration_fee: u128,
    pub total_registration_fees: u128,
    
    pub leftover_withdrawn: bool,
    pub tokens_deposited: bool,
    pub parameters_set: bool 
}

impl Distributor {
    pub fn register(&mut self) {
        require!(self.registration_fee == msg::value(), "Registration deposit doesn't match");

        self.total_registration_fees = self.total_registration_fees.saturating_add(msg::value());

        self.register_user(msg::source());
    }

    pub fn register_multiple(&mut self, users: Vec<ActorId>) {
        self.only_admin();

        users.into_iter()
            .for_each(|user| {
                if !self.was_user_registered(&user) {
                    self.register_user(user); 
                }
            });
    }

    pub fn participate(&mut self) {
        self.only_if_distribution_is_not_over();
        require!(self.was_user_registered(&msg::source()), "Address must be registered on distribution");
        require!(!self.was_user_participated(&msg::source()), "Address already participated");

        self.distribution.users.insert(msg::source(), false);

        msg::reply(DistributorEvent::Participated {
            who: msg::source(),
            when: exec::block_timestamp()
        }, 0).unwrap();
    }

    pub async fn withdraw(&mut self) {
        require!(
            self.vesting.portions_unlock_times.len() > 0 &&
            self.vesting.percents_per_portions.len() > 0,
            "Vesting parameters are not set"
        );
        require!(self.was_user_registered(&msg::source()), "Address is not registered");
        require!(self.was_user_participated(&msg::source()), "Address is not participated in distribution");
        require!(self.was_user_claimed(&msg::source()), "Address has executed withdraw already");

        let allocation = *self.registration.users
            .get(&msg::source())
            .unwrap();

        require!(allocation > 0, "There is nothing to withdraw");

        let mut to_widthaw: u128 = 0;
        for (index, _time) in self.vesting.portions_unlock_times.iter().enumerate() {
            if exec::block_timestamp() >= self.vesting.portions_unlock_times[index] {
                let amount_withdrawing = allocation
                    .saturating_mul(self.vesting.percents_per_portions[index])
                    .saturating_div(self.vesting.precision);

                to_widthaw = to_widthaw.saturating_add(amount_withdrawing);
            }
        }

        self.distribution.users
            .entry(msg::source())
            .and_modify(|withdrawn| { *withdrawn = true });

        self.total_tokens_distributed = self.total_tokens_distributed.saturating_add(to_widthaw);

        transfer_tokens(
            &self.distribution_parameters.token, 
            &exec::program_id(), 
            &msg::source(), 
            to_widthaw)
            .await;

        msg::reply(DistributorEvent::Withdrawn(exec::block_timestamp()), 0).expect("reply: 'Withdrawn' error");
    }

    pub fn set_registration_fee(&mut self, amount: u128) {
        self.only_admin();
        self.registration_fee = amount;

        msg::reply(DistributorEvent::RegistrationFeeSet(exec::block_timestamp()), 0).unwrap();
    }

    pub fn set_registration_time(&mut self, start_datetime: u64, end_datetime: u64) {
        self.only_admin();

        require!(
            start_datetime >= exec::block_timestamp() && end_datetime > start_datetime, 
            "Registration's start date must be in future");

        self.registration = RegistrationRound {
            start_datetime,
            end_datetime,
            ..Default::default()
        };

        msg::reply(DistributorEvent::RegistrationRoundSet(exec::block_timestamp()), 0).unwrap();
    }

    pub fn set_distribution_time(&mut self, start_datetime: u64, end_datetime: u64) {
        self.only_admin();
        
        require!(self.parameters_set, "Distribution parameters are not set");
        require!(
            start_datetime >= exec::block_timestamp() && end_datetime > start_datetime, 
            "Distribution's start date must be in future");
        require!(
            start_datetime >= self.registration.end_datetime, 
            "Distribution round must be later than registration round");

        self.distribution = DistributionRound {
            start_datetime,
            end_datetime,
            ..Default::default()
        };

        msg::reply(DistributorEvent::DistributionRoundSet(exec::block_timestamp()), 0).unwrap();
    }

    pub fn set_distribution_parameters(
        &mut self,
        token: ActorId,
        owner: ActorId,
        amount_of_tokens_to_distribute: u128,
        vesting_precision: u128
        ) {
        require!(!self.parameters_set, "Distribution parameters already set");

        self.distribution_parameters = DistributionParameters {
            token,
            owner,
            amount_of_tokens_to_distribute
        };

        self.vesting = VestingParameters {
            precision: vesting_precision,
            ..Default::default()
        };

        self.parameters_set = true;

        msg::reply(DistributorEvent::DistributionParametersSet(exec::block_timestamp()), 0).unwrap();
    }

    pub fn set_vesting_parameters(
        &mut self,
        portions_unlock_times: Vec<u64>,
        percents_per_portions: Vec<u128>
    ) {
        require!(self.parameters_set, "Distribution parameters are not set");
        require!(
            self.vesting.portions_unlock_times.len() == 0 &&
            self.vesting.percents_per_portions.len() == 0,
            "Vesting parameters already set"
        );
        require!(
            portions_unlock_times.len() == percents_per_portions.len(),
            "Unlocking Times length must be equal with Percent Per Portion length"
        );
        require!(
            *portions_unlock_times.last().unwrap() > self.distribution.end_datetime,
            "Unlock time must be after the distribution ends"
        );

        let precision = 0;
        for (index, time) in portions_unlock_times.iter().enumerate() {
            if index > 0 {
                require!(
                    portions_unlock_times[index] > portions_unlock_times[index - 1], 
                    "Unlock times dates issue");
            }

            self.vesting.portions_unlock_times.push(*time);
            self.vesting.percents_per_portions.push(percents_per_portions[index]);

            self.vesting.precision = self.vesting.precision.saturating_add(percents_per_portions[index]); 
        }
        
        require!(self.vesting.precision == precision, "Precision percents issue");

        msg::reply(DistributorEvent::VestingParametersSet(exec::block_timestamp()), 0).unwrap();
    }

    pub fn set_allocation_size(&mut self, user: ActorId, amount: u128) {
        self.only_admin();
        require!(self.was_user_registered(&user), "Address is not registered");

        self.registration.users
            .entry(msg::source())
            .and_modify(|allocation| { *allocation += amount });
    }

    pub async fn deposit_tokens(&mut self) {
        self.only_owner();
        require!(self.parameters_set, "Distribution parameters are not set");
        require!(!self.tokens_deposited, "Tokens has been deposited already");

        self.tokens_deposited = true;

        transfer_tokens(
            &self.distribution_parameters.token, 
            &msg::source(), 
            &exec::program_id(), 
            self.distribution_parameters.amount_of_tokens_to_distribute)
            .await;

        msg::reply(DistributorEvent::TokensDeposited(exec::block_timestamp()), 0).expect("reply: 'TokensDeposited' error");
    }

    pub fn set_vesting_enddate(&mut self, enddate: u64) {
        self.vesting_enddate = enddate;
    }

    pub async fn withdraw_leftover(&mut self) {
        self.only_admin();

        require!(self.vesting_enddate > 0, "Vesting end date is not set");
        require!(exec::block_timestamp() >= self.vesting_enddate, "Vesting period is not finished yet");
        require!(!self.leftover_withdrawn, "Leftover already withdrawn");

        let leftover = self.distribution_parameters
            .amount_of_tokens_to_distribute.saturating_sub(self.total_tokens_distributed);

        require!(leftover > 0, "There is nothing to withdraw");
        
        self.leftover_withdrawn = true;
        
        transfer_tokens(
            &self.distribution_parameters.token, 
            &exec::program_id(), 
            &msg::source(), 
            leftover)
            .await;

        msg::reply(DistributorEvent::LeftoverWithdrawn(exec::block_timestamp()), 0).expect("reply: 'LeftoverWithdrawn' error");
    }

    pub async fn withdraw_fee(&mut self) {
        self.only_admin();

        require!(exec::block_timestamp() >= self.registration.end_datetime, "Registration round is not over yet");
        require!(self.total_registration_fees > 0, "There are no tokens to withdraw");

        msg::reply(DistributorEvent::FeeWithdrawn(self.total_registration_fees), self.total_registration_fees).unwrap();

        self.total_registration_fees = 0;
    }

    pub fn get_registered_users(&self) -> Vec<ActorId> {
        self.registration.users
            .clone()
            .into_iter()
            .map(|user| { 
                let (address, _allocation) = user;

                address
            })
            .collect()
    }

    pub fn get_participated_users(&self) -> Vec<ActorId> {
        self.distribution.users
            .clone()
            .into_iter()
            .map(|user| { 
                let (address, _withdrawn) = user;

                address
            })
            .collect()
    }

    pub fn get_claimed_users(&self) -> Vec<ActorId> {
        self.distribution.users
            .clone()
            .into_iter()
            .filter(|user| {
                let withdrawn = user.1;
                
                withdrawn == true
            })
            .map(|user| { 
                let (address, _withdrawn) = user;

                address
            })
            .collect()
    }

    fn register_user(&mut self, address: ActorId) {
        self.only_if_registration_is_not_over();
        require!(!self.was_user_registered(&address), "Address already registered");
        
        self.registration.users.insert(address, 0);

        msg::reply(DistributorEvent::Registered {
            who: address,
            when: exec::block_timestamp()
        }, 0).unwrap();
    }

    fn was_user_registered(&self, user: &ActorId) -> bool {
        self.registration.users.contains_key(user)
    }

    fn was_user_participated(&self, user: &ActorId) -> bool {
        self.distribution.users.contains_key(user)
    }

    fn was_user_claimed(&self, user: &ActorId) -> bool {
        let user = self.distribution.users.get(user);
        match user {
            Some(claimed) => *claimed,
            None => false,
        }
    }

    fn only_admin(&self) {
        require!(self.admin == msg::source(), "Allows only admin address");
    }

    fn only_owner(&self) {
        require!(
            self.distribution_parameters.owner == msg::source(),
             "Allows only owner address"
            );
    }

    fn only_if_registration_is_not_over(&self) {
        require!(exec::block_timestamp() < self.registration.end_datetime, "Registration round is over or not started yet'");
    }

    fn only_if_distribution_is_not_over(&self) {
        require!(exec::block_timestamp() < self.distribution.end_datetime, "Distribution round is over or not started yet'");
    }
}

async fn transfer_tokens(
    token_address: &ActorId,
    from: &ActorId,
    to: &ActorId,
    amount: u128,
) {
    msg::send_for_reply(
        *token_address,
        FTAction::Transfer {
            from: *from,
            to: *to,
            amount,
        },
        0,
    )
    .expect("Distributor: error in sending message")
    .await
    .expect("Distributor: error in transfer");
}