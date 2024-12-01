use ic_cdk::{query, update};

// Everyone should be able to call this
// the tx.from == caller otherwise tx should not be added
// Validations should be done by calling cketh minter to make sure transaction exsits
#[update]
fn new_icp_to_evm_tx() {}

// Everyone should be able to call this
// Validation Should be done on a timer basis and if tx does not exist
// Transaction should be removed
#[update]
fn new_evm_to_icp_tx() {}

fn main() {}
