use std::time::Duration;

pub mod add_evm_tokens;
pub mod address;
pub mod appic_dex_types;
pub mod cbor;
pub mod checked_amount;
pub mod endpoints;
pub mod guard;
pub mod icp_tokens_service;
pub mod ledger_manager_client;
pub mod lifecycle;
pub mod logs;
pub mod minter_client;
pub mod numeric;
pub mod remove_unverified_tx;
pub mod scrape_events;
pub mod state;
pub mod update_bridge_pairs;
pub mod update_icp_tokens;

#[cfg(test)]
pub mod tests;

// 1 Minute
pub const SCRAPE_EVENTS: Duration = Duration::from_secs(1 * 60);

// 5 Minutes
pub const UPDATE_USD_PRICE: Duration = Duration::from_secs(10 * 60);

// 1 Day
pub const REMOVE_UNVERIFIED_TX: Duration = Duration::from_secs(24 * 60 * 60);

// 1 Day
pub const UPDATE_BRIDGE_PAIRS: Duration = Duration::from_secs(24 * 60 * 60);

// 1 Day
pub const UPDATE_ICP_TOKENS: Duration = Duration::from_secs(24 * 60 * 60);

// 1 Week
pub const REMOVE_INVALID_ICP_TOKENS: Duration = Duration::from_secs(7 * 24 * 60 * 60);
