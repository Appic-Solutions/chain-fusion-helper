use crate::{
    minter_clinet::MinterClient,
    state::{mutate_state, read_state, Oprator},
};

use crate::minter_clinet::event_conversion::Events;
const MAX_EVENTS_PER_RESPONSE: u64 = 100;

pub async fn update_latest_events_count() {
    let minters_iter = read_state(|s| s.get_minters_iter());

    for (minter_key, minter) in minters_iter {
        let mitner_client = MinterClient::from(&minter);

        // Get the latest event count to update last_observed_event;
        let latest_event_count = mitner_client.get_total_events_count().await;

        // Check if the previos last_observed_event is greater or equal to latest one;
        // If yes there should be no scraping for events and last_observed_event should not be updated
        if minter.last_observed_event >= latest_event_count {
            break;
        };

        // Updating process
        mutate_state(|s| {
            if let Some(muted_minter) = s.get_minter_mut(&minter_key) {
                muted_minter.update_last_observed_event(latest_event_count);
            }
        });
    }
}

pub async fn scrape_events_range() {}

fn apply_state_transition(events: Events, oprator: Oprator) {
    for event in events.events.iter() {}
}
