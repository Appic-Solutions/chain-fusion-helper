use crate::{
    appic_dex_client::DexClient,
    appic_dex_types::GetEventsResult,
    guard::TimerGuard,
    logs::{DEBUG, INFO},
    state::{mutate_state, read_state},
};

use ic_canister_log::log;

const MAX_EVENTS_PER_RESPONSE: u64 = 100;

pub const NATIVE_ERC20_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

pub async fn scrape_dex_events() {
    // Issue a timer guard
    let _guard = match TimerGuard::new(crate::guard::TaskType::ScrapeDexEvents) {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let dex_info = read_state(|s| s.dex_info.get().clone());

    let dex_clinet = DexClient::new(dex_info.id);

    // Get the latest event count to update last_observed_event;
    // -1 since the starting index in 0 not 1
    let latest_event_count = dex_clinet.get_total_events_count().await - 1;

    // Check if the previous last_observed_event is greater or equal to latest one;
    // If yes there should be no scraping for events and last_observed_event should not be updated
    if dex_info.last_observed_event >= latest_event_count {
        return;
    };

    // Updating last observed event count
    mutate_state(|s| s.update_last_observed_dex_event(latest_event_count));

    let last_scraped_event = dex_info.last_scraped_event;

    // Scraping logs between specified ranges
    // MAX_EVENT_RESPONSE= 100 so the log range should not be more than 100
    // min((last_observed_event - last_scraped_event),100) will be the specified range
    // If last_observed_event - last_scraped_event contains more than 100, the event scaping will be divided into multiple calls

    scrape_events_range(
        latest_event_count,
        last_scraped_event,
        MAX_EVENTS_PER_RESPONSE,
        &dex_clinet,
    )
    .await
}

pub async fn scrape_events_range(
    last_observed_event: u64,
    last_scraped_event: u64,
    max_event_scrap: u64,
    dex_client: &DexClient,
) {
    if last_scraped_event >= last_observed_event {
        log!(
            INFO,
            "[Scraping Events DEX] No events to scrape. All events are already processed."
        );
        return;
    }

    let mut start = last_scraped_event; // Start from the next event after the last scraped
    let end = last_observed_event; // Scrape up to the last observed event
    const MAX_RETRIES: u32 = 5; // Maximum retry attempts

    while start <= end {
        let chunk_end = std::cmp::min(start + max_event_scrap - 1, end); // Define the range limit
        log!(
            INFO,
            "[Scraping Events DEX] Scraping events from {} to {}",
            start,
            chunk_end,
        );

        let mut attempts = 0; // Initialize retry counter
        let mut success = false; // Track success status

        while attempts < MAX_RETRIES {
            let events_result = dex_client.scrape_events(start, 100).await;
            match events_result {
                Ok(events) => {
                    log!(INFO, "[Scraping Events] Received Event {:?}", events);

                    apply_dex_state_transition(events);
                    mutate_state(|s| s.update_last_scraped_dex_event(chunk_end));
                    success = true; // Mark as successful
                    break; // Exit retry loop
                }
                Err(err) => {
                    attempts += 1;
                    log!(
                        DEBUG,
                        "[Scraping Events DEX] Error scraping events from {} to {}: {:?}. Retrying... ({}/{})",
                        start,
                        chunk_end,
                        err,
                        attempts,
                        MAX_RETRIES
                    );

                    if attempts >= MAX_RETRIES {
                        log!(
                            DEBUG,
                            "[Scraping Events DEX] Failed to scrape events from {} to {} after {} retries. Skipping...",
                            start,
                            chunk_end,
                            MAX_RETRIES
                        );
                    }
                }
            }
        }

        if success {
            // Move to the next range only if scraping was successful
            start = chunk_end + 1;
        } else {
            // If scraping ultimately fails, break to prevent an infinite loop
            log!(
                DEBUG,
                "[Scraping Events DEX] Aborting further scraping due to repeated failures."
            );
            break;
        }
    }
}

pub fn apply_dex_state_transition(events: GetEventsResult) {
    for event in events.events.into_iter() {
        let principal = event.payload.get_principal();
        mutate_state(|s| s.record_dex_action_for_principal(principal, event.into()))
    }
}
