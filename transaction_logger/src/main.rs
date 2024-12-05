use ic_canister_log::log;
use ic_cdk::{init, post_upgrade, query, update};
use ic_cdk_timers;
use transaction_logger::lifecycle;
use transaction_logger::{
    endpoints::LoggerArgs,
    logs::INFO,
    remove_unverified_tx::remove_unverified_tx,
    scrape_events::scrape_events,
    state::{init_state, State},
    update_token_pairs::update_token_pairs,
    CHECK_NEW_ICRC_TWIN_TOKENS, REMOVE_UNVERIFIED_TX_INTERVAL, SCRAPE_EVENTS_INTERVAL,
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

#[init]
pub fn init(init_args: LoggerArgs) {
    match init_args {
        LoggerArgs::Init(init_args) => {
            log!(INFO, "[init]: initialized minter with arg: {:?}", init_args);
            let state = State::from(init_args);
            init_state(state);
        }
        LoggerArgs::Upgrade(_upgrade_arg) => {
            ic_cdk::trap("cannot init canister state with upgrade args");
        }
    }

    setup_timers();
}

#[post_upgrade]
fn post_upgrade(upgrade_args: Option<LoggerArgs>) {
    // Upgrade necessary parts if needed

    match upgrade_args {
        Some(LoggerArgs::Init(_)) => {
            ic_cdk::trap("cannot upgrade canister state with init args");
        }
        Some(LoggerArgs::Upgrade(upgrade_args)) => lifecycle::post_upgrade(Some(upgrade_args)),
        None => lifecycle::post_upgrade(None),
    }

    // Set up timers
    setup_timers();
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
