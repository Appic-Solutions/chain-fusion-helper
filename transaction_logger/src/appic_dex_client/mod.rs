use candid::Principal;

use crate::appic_dex_types::{GetEventsArg, GetEventsResult};
use crate::minter_client::{CallError, IcRunTime, Runtime};
pub struct DexClient {
    runtime: IcRunTime,
    dex_id: Principal,
}

impl DexClient {
    pub fn new(dex_id: Principal) -> Self {
        Self {
            runtime: IcRunTime(),
            dex_id,
        }
    }

    // Get total events count
    pub async fn get_total_events_count(&self) -> u64 {
        // Get total events count
        let total_events_count = self
            .runtime
            .call_canister::<GetEventsArg, GetEventsResult>(
                self.dex_id,
                "get_events",
                GetEventsArg {
                    start: 0,
                    length: 0,
                },
            )
            .await
            .expect("Call should not fail. will retry in next interval")
            .total_event_count;

        total_events_count
    }

    // scrape events
    pub async fn scrape_events(
        &self,
        from_event: u64,
        length: u64,
    ) -> Result<GetEventsResult, CallError> {
        self.runtime
            .call_canister::<GetEventsArg, GetEventsResult>(
                self.dex_id,
                "get_events",
                GetEventsArg {
                    start: from_event,
                    length,
                },
            )
            .await
    }
}
