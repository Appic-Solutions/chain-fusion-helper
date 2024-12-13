use crate::{icp_tokens_service::TokenService, logs::INFO};
use ic_canister_log::log;
pub async fn update_icp_tokens() {
    let icp_swap_tokens = TokenService::new().get_icp_swap_tokens().await;
    let sonic_swap_tokens = TokenService::new().get_sonic_tokens().await;

    log!(
        INFO,
        "[Scrape new twin tokens] Start scraping new twin tokens from appic lsm",
    );
}
