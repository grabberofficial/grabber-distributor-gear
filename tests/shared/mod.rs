use ft_io::*;
use gstd::{ActorId, Encode};
use gtest::{Program, System};
use hashbrown::HashMap;
use polkapad_distributor::io::*;

pub const DISTRIBUTOR_ADDRESS: u64 = 1;
pub const DEPLOYER: u64 = 2;
pub const OWNER: u64 = 3;

pub const ADMIN: u64 = 10;
pub const ALICE: u64 = 11;

pub const TOKENS_TO_DISTRIBUTE: u128 = 100_000_000 * 10e18 as u128;
pub const DECIMALS: u32 =  18;

fn init_token(system: &System, name: &str, symbol: &str) {
    let token = Program::from_file(system, "../fungible-token/target/wasm32-unknown-unknown/release/fungible_token.wasm");

    let result = token.send(
        DEPLOYER,
        InitConfig {
            name: String::from(name),
            symbol: String::from(symbol),
            decimals: DECIMALS as u8
        },
    );

    assert!(!result.main_failed());

    token.send(DEPLOYER, FTAction::Transfer { 
        from: DEPLOYER.into(),
        to: OWNER.into(), 
        amount: TOKENS_TO_DISTRIBUTE 
    });
}

pub fn init_distributor(system: &System) {
    let distributor = Program::from_file(system, "./target/wasm32-unknown-unknown/release/polkapad_distributor.wasm");

    let result = distributor.send(
        ADMIN,
        InitDistributor {},
    );

    assert!(!result.main_failed());
}

pub fn set_registration_round(distributor: &Program, start_date: u64, end_date: u64) {
    distributor.send(ADMIN, DistributorAction::SetRegistrationRound {
        start_datetime: start_date,
        end_datetime: end_date
    });
}

pub fn set_distribution_round(distributor: &Program, start_date: u64, end_date: u64) {
    distributor.send(ADMIN, DistributorAction::SetDistributionRound {
        start_datetime: start_date,
        end_datetime: end_date
    });
}

pub fn transfer_tokens(system: &System, token: u64, from: u64, to: u64, amount: u128) {
    let token = system.get_program(token);

    token.send(DEPLOYER, FTAction::Transfer { 
        from: from.into(), 
        to: to.into(), 
        amount 
    });
}