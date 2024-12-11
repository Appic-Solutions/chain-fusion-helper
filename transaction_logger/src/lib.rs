use std::time::Duration;

pub mod checked_amount;
pub mod endpoints;
pub mod guard;
mod ledger_manager_client;
pub mod lifecycle;
pub mod logs;
pub mod minter_clinet;
pub mod numeric;
pub mod remove_unverified_tx;
pub mod scrape_events;
pub mod state;
pub mod update_token_pairs;

pub const SCRAPE_EVENTS_INTERVAL: Duration = Duration::from_secs(1 * 60);

pub const REMOVE_UNVERIFIED_TX_INTERVAL: Duration = Duration::from_secs(60 * 60);

pub const CHECK_NEW_ICRC_TWIN_TOKENS: Duration = Duration::from_secs(60 * 60);
