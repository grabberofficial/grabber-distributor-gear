use gtest::{System};
use gstd::{Encode, String};

use polkapad_distributor::io::*;

mod shared;
use shared::*;

#[test]
fn register_user_should_registered() {
    let system = System::new();
    init_distributor(&system);

    let distributor = system.get_program(DISTRIBUTOR_ADDRESS);

    let registration_start_date = system.block_timestamp();
    let registration_end_date = system.block_timestamp() + 20000;
    set_registration_round(&distributor, registration_start_date, registration_end_date);

    let result = distributor.send(ALICE, DistributorAction::Register);
    assert!(result.contains(&(ALICE, DistributorEvent::Registered { 
        who: ALICE.into(),
        when: system.block_timestamp()
     }.encode())));
}

#[test]
fn register_user_twice_should_failed() {
    let system = System::new();
    init_distributor(&system);

    let distributor = system.get_program(DISTRIBUTOR_ADDRESS);

    let registration_start_date = system.block_timestamp();
    let registration_end_date = system.block_timestamp() + 20000;

    distributor.send(ADMIN, DistributorAction::SetRegistrationRound {
        start_datetime: registration_start_date,
        end_datetime: registration_end_date
    });

    distributor.send(ALICE, DistributorAction::Register);
    let result = distributor.send(ALICE, DistributorAction::Register);
    assert!(result.main_failed());
}

#[test]
fn register_user_with_deposit_should_registered() {
    let system = System::new();
    init_distributor(&system);
    
    system.mint_to(ALICE, 1500);

    let distributor = system.get_program(DISTRIBUTOR_ADDRESS);

    let registration_fee: u128 = 1000;
    let registration_start_date = system.block_timestamp();
    let registration_end_date = system.block_timestamp() + 20000;

    distributor.send(ADMIN, DistributorAction::SetRegistrationFee(registration_fee));

    distributor.send(ADMIN, DistributorAction::SetRegistrationRound {
        start_datetime: registration_start_date,
        end_datetime: registration_end_date
    });

    let result = distributor.send_with_value(ALICE, DistributorAction::Register, registration_fee);
    assert!(result.contains(&(ALICE, DistributorEvent::Registered { 
        who: ALICE.into(),
        when: system.block_timestamp()
     }.encode())));
}

#[test]
fn register_user_with_incorrect_deposit_should_failed() {
    let system = System::new();
    init_distributor(&system);
    
    system.mint_to(ALICE, 1100);

    let distributor = system.get_program(DISTRIBUTOR_ADDRESS);

    let registration_fee: u128 = 1000;
    let registration_start_date = system.block_timestamp();
    let registration_end_date = system.block_timestamp() + 20000;


    distributor.send(ADMIN, DistributorAction::SetRegistrationFee(registration_fee));

    distributor.send(ADMIN, DistributorAction::SetRegistrationRound {
        start_datetime: registration_start_date,
        end_datetime: registration_end_date
    });

    let result = distributor.send_with_value(ALICE, DistributorAction::Register, 1100);
    assert!(result.main_failed());
}