use std::time::Duration;

pub mod add_evm_tokens;
pub mod checked_amount;
pub mod endpoints;
pub mod guard;
pub mod icp_tokens_service;
pub mod ledger_manager_client;
pub mod lifecycle;
pub mod logs;
pub mod minter_clinet;
pub mod numeric;
pub mod remove_unverified_tx;
pub mod scrape_events;
pub mod state;
pub mod update_icp_tokens;
pub mod update_token_pairs;

// 1 Minute
pub const SCRAPE_EVENTS_INTERVAL: Duration = Duration::from_secs(1 * 60);

// 1 Hour
pub const REMOVE_UNVERIFIED_TX_INTERVAL: Duration = Duration::from_secs(60 * 60);

// 1 Hour
pub const CHECK_NEW_ICRC_TWIN_TOKENS: Duration = Duration::from_secs(60 * 60);

// 3 Days
pub const UPDATE_ICP_TOKENS: Duration = Duration::from_secs(3 * 24 * 60 * 60);
