use gtest::{System};
use gstd::{Encode, String, exec::block_height};

use polkapad_distributor::io::*;

mod shared;
use shared::*;

#[test]
fn participate_should_participated() {
    let system = System::new();
    init_distributor(&system);

    let distributor = system.get_program(DISTRIBUTOR_ADDRESS);

    let registration_start_date = system.block_timestamp();
    let registration_end_date = system.block_timestamp() + 5000;
    set_registration_round(&distributor, registration_start_date, registration_end_date);

    let distribution_start_date = registration_end_date + 5000;
    let distribution_end_date = registration_end_date + 10000;
    set_distribution_round(&distributor, distribution_start_date, distribution_end_date);

    distributor.send(ALICE, DistributorAction::Register);
    
    system.spend_blocks(11);

    let result = distributor.send(ALICE, DistributorAction::Participate);
    assert!(!result.main_failed());
}