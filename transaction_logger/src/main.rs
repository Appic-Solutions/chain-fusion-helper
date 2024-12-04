use ic_cdk::{query, update};
use ic_cdk_timers;
use transaction_logger::{
    remove_unverified_tx::remove_unverified_tx, scrape_events::scrape_events,
    update_token_pairs::update_token_pairs, CHECK_NEW_ICRC_TWIN_TOKENS,
    REMOVE_UNVERIFIED_TX_INTERVAL, SCRAPE_EVENTS_INTERVAL,
};
// Setup timers
fn setup_timers() {
    // Start scraping events.
    ic_cdk_timers::set_timer_interval(SCRAPE_EVENTS_INTERVAL, || ic_cdk::spawn(scrape_events()));

    // Remove unverified transactions
    ic_cdk_timers::set_timer_interval(REMOVE_UNVERIFIED_TX_INTERVAL, || remove_unverified_tx());

    // Check new supported twin tokens
    ic_cdk_timers::set_timer_interval(CHECK_NEW_ICRC_TWIN_TOKENS, || {
        ic_cdk::spawn(update_token_pairs())
    });
}

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
