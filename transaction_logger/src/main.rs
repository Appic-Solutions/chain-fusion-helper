use ic_cdk::{query, update};

// Everyone should be able to call this
// the tx.from == caller otherwise tx should not be added
// Validations should be done by calling cketh minter to make sure transaction exsits
#[update]
fn new_icp_to_evm_ck_tx() {}

// Only appic minters should be able to call this
// Transactions will automatically be added.
#[update]
fn new_icp_to_evm_appic_tx() {}

// Everyone should be able to call this
// Validation Should be done on a timer basis and if tx does not exist
// Transaction should be removed
#[update]
fn new_evm_to_icp_ck_tx() {}

// Everyone should be able to call this
// If the minter does not update after specific time tx should be removed.
#[update]
fn new_evm_to_icp_appic_tx() {}
fn main() {}
