use crate::{
    guard::TimerGuard,
    logs::{DEBUG, INFO},
    minter_clinet::MinterClient,
    numeric::Erc20TokenAmount,
    state::{
        mutate_state, nat_to_ledger_burn_index, nat_to_ledger_mint_index, read_state, ChainId,
        EvmToIcpTxIdentifier, IcpToEvmIdentifier, MinterKey, Operator,
    },
};

use crate::minter_clinet::appic_minter_types::events::EventPayload as AppicEventPayload;
use ic_canister_log::log;

use crate::minter_clinet::event_conversion::Events;
const MAX_EVENTS_PER_RESPONSE: u64 = 100;

pub const NATIVE_ERC20_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

pub async fn scrape_events() {
    // Issue a timer gaurd
    let _gaurd = match TimerGuard::new(crate::guard::TaskType::ScrapeEvents) {
        Ok(gaurd) => gaurd,
        Err(_) => return,
    };

    let minters = read_state(|s| s.get_minters());

    for (minter_key, minter) in minters.iter() {
        let minter_client = MinterClient::from(minter);

        // Get the latest event count to update last_observed_event;
        // -1 since the starting index in 0 not 1
        let latest_event_count = minter_client.get_total_events_count().await - 1;

        // Check if the previos last_observed_event is greater or equal to latest one;
        // If yes there should be no scraping for events and last_observed_event should not be updated
        if minter.last_observed_event >= latest_event_count {
            break;
        };

        // Updating last observed event count
        mutate_state(|s| s.update_last_observed_event(minter_key, latest_event_count));

        let last_scraped_event = minter.last_scraped_event;

        // Scraping logs between specified ranges
        // MAX_EVENT_RESPONSE= 100 so the log range should not be more than 100
        // min((last_observed_evnet - last_scraped_event),100) will be the specified range
        // If last_observed_evnet - last_scraped_event contains more than 100, the event scaping will be divided into multiple calls

        scrape_events_range(
            latest_event_count,
            last_scraped_event,
            MAX_EVENTS_PER_RESPONSE,
            &minter_client,
            minter_key,
            minter.evm_to_icp_fee,
            minter.icp_to_evm_fee,
        )
        .await
    }
}

pub async fn scrape_events_range(
    last_observed_event: u64,
    last_scraped_event: u64,
    max_event_scrap: u64,
    minter_client: &MinterClient,
    minter_key: &MinterKey,
    evm_to_icp_fee: Erc20TokenAmount,
    icp_to_evm_fee: Erc20TokenAmount,
) {
    if last_scraped_event >= last_observed_event {
        log!(
            INFO,
            "[Scraping Events] No events to scrape. All events are already processed."
        );
        return;
    }

    let mut start = last_scraped_event + 1; // Start from the next event after the last scraped
    let end = last_observed_event; // Scrape up to the last observed event
    const MAX_RETRIES: u32 = 5; // Maximum retry attempts

    while start <= end {
        let chunk_end = std::cmp::min(start + max_event_scrap - 1, end); // Define the range limit
        log!(
            INFO,
            "[Scraping Events] Scraping events from {} to {} minter {:?}",
            start,
            chunk_end,
            minter_key
        );

        let mut attempts = 0; // Initialize retry counter
        let mut success = false; // Track success status

        while attempts < MAX_RETRIES {
            let events_result = minter_client.scrape_events(start, 100).await;
            match events_result {
                Ok(events) => {
                    apply_state_transition(
                        events,
                        minter_key.operator(),
                        minter_key.chain_id(),
                        evm_to_icp_fee,
                        icp_to_evm_fee,
                    );
                    mutate_state(|s| s.update_last_scraped_event(&minter_key, chunk_end));
                    success = true; // Mark as successful
                    break; // Exit retry loop
                }
                Err(err) => {
                    attempts += 1;
                    log!(
                        DEBUG,
                        "[Scraping Events] Error scraping events from {} to {}: {:?}. Retrying... ({}/{})",
                        start,
                        chunk_end,
                        err,
                        attempts,
                        MAX_RETRIES
                    );

                    if attempts >= MAX_RETRIES {
                        log!(
                            DEBUG,
                            "[Scraping Events] Failed to scrape events from {} to {} after {} retries. Skipping...",
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
                "[Scraping Events] Aborting further scraping due to repeated failures."
            );
            break;
        }
    }
}

fn apply_state_transition(
    events: Events,
    operator: Operator,
    chain_id: ChainId,
    evm_to_icp_fee: Erc20TokenAmount,
    icp_to_evm_fee: Erc20TokenAmount,
) {
    for event in events.events.into_iter() {
        // Applying the state transition
        mutate_state(|s| match event.payload {
            AppicEventPayload::Init(_init_arg) => {}
            AppicEventPayload::Upgrade(_upgrade_arg) => {}
            AppicEventPayload::AcceptedDeposit {
                transaction_hash,
                block_number,
                from_address,
                value,
                principal,
                subaccount,
                ..
            } => s.record_accepted_evm_to_icp(
                EvmToIcpTxIdentifier::new(&transaction_hash, chain_id),
                transaction_hash,
                block_number,
                from_address,
                value,
                principal,
                NATIVE_ERC20_ADDRESS.to_string(),
                subaccount,
                chain_id,
                operator,
                event.timestamp,
            ),
            AppicEventPayload::AcceptedErc20Deposit {
                transaction_hash,
                block_number,
                log_index: _,
                from_address,
                value,
                principal,
                erc20_contract_address,
                subaccount,
            } => s.record_accepted_evm_to_icp(
                EvmToIcpTxIdentifier::new(&transaction_hash, chain_id),
                transaction_hash,
                block_number,
                from_address,
                value,
                principal,
                erc20_contract_address,
                subaccount,
                chain_id,
                operator,
                event.timestamp,
            ),
            AppicEventPayload::InvalidDeposit {
                event_source,
                reason,
            } => s.record_invalid_evm_to_icp(
                EvmToIcpTxIdentifier::new(&event_source.transaction_hash, chain_id),
                reason,
            ),
            AppicEventPayload::MintedNative {
                event_source,
                mint_block_index,
            } => s.record_minted_evm_to_icp(
                EvmToIcpTxIdentifier::new(&event_source.transaction_hash, chain_id),
                evm_to_icp_fee,
                nat_to_ledger_mint_index(&mint_block_index),
            ),
            AppicEventPayload::SyncedToBlock { .. } => {}
            AppicEventPayload::AcceptedNativeWithdrawalRequest {
                withdrawal_amount,
                destination,
                ledger_burn_index,
                from,
                from_subaccount,
                created_at,
            } => s.record_accepted_icp_to_evm(
                IcpToEvmIdentifier::new(nat_to_ledger_burn_index(&ledger_burn_index), chain_id),
                None,
                withdrawal_amount,
                NATIVE_ERC20_ADDRESS.to_string(),
                destination,
                ledger_burn_index,
                None,
                from,
                from_subaccount,
                created_at,
                operator,
                chain_id,
                event.timestamp,
            ),
            AppicEventPayload::CreatedTransaction { withdrawal_id, .. } => s
                .record_created_icp_to_evm(IcpToEvmIdentifier::new(
                    nat_to_ledger_burn_index(&withdrawal_id),
                    chain_id,
                )),
            AppicEventPayload::SignedTransaction { withdrawal_id, .. } => s
                .record_signed_icp_to_evm(IcpToEvmIdentifier::new(
                    nat_to_ledger_burn_index(&withdrawal_id),
                    chain_id,
                )),
            AppicEventPayload::ReplacedTransaction { withdrawal_id, .. } => s
                .record_replaced_icp_to_evm(IcpToEvmIdentifier::new(
                    nat_to_ledger_burn_index(&withdrawal_id),
                    chain_id,
                )),
            AppicEventPayload::FinalizedTransaction {
                withdrawal_id,
                transaction_receipt,
            } => s.record_finalized_icp_to_evm(
                IcpToEvmIdentifier::new(nat_to_ledger_burn_index(&withdrawal_id), chain_id),
                transaction_receipt,
                icp_to_evm_fee,
            ),
            AppicEventPayload::ReimbursedNativeWithdrawal { withdrawal_id, .. } => s
                .record_reimbursed_icp_to_evm(IcpToEvmIdentifier::new(
                    nat_to_ledger_burn_index(&withdrawal_id),
                    chain_id,
                )),
            AppicEventPayload::ReimbursedErc20Withdrawal { withdrawal_id, .. } => s
                .record_reimbursed_icp_to_evm(IcpToEvmIdentifier::new(
                    nat_to_ledger_burn_index(&withdrawal_id),
                    chain_id,
                )),
            AppicEventPayload::SkippedBlock { .. } => {}
            AppicEventPayload::AddedErc20Token { .. } => {}
            AppicEventPayload::AcceptedErc20WithdrawalRequest {
                max_transaction_fee,
                withdrawal_amount,
                erc20_contract_address,
                destination,
                native_ledger_burn_index,
                erc20_ledger_burn_index,
                from,
                from_subaccount,
                created_at,
                ..
            } => s.record_accepted_icp_to_evm(
                IcpToEvmIdentifier::new(
                    nat_to_ledger_burn_index(&native_ledger_burn_index),
                    chain_id,
                ),
                Some(max_transaction_fee),
                withdrawal_amount,
                erc20_contract_address,
                destination,
                native_ledger_burn_index,
                Some(erc20_ledger_burn_index),
                from,
                from_subaccount,
                Some(created_at),
                operator,
                chain_id,
                event.timestamp,
            ),
            AppicEventPayload::FailedErc20WithdrawalRequest { withdrawal_id, .. } => s
                .record_reimbursed_icp_to_evm(IcpToEvmIdentifier::new(
                    nat_to_ledger_burn_index(&withdrawal_id),
                    chain_id,
                )),
            AppicEventPayload::MintedErc20 {
                event_source,
                mint_block_index,
                ..
            } => s.record_minted_evm_to_icp(
                EvmToIcpTxIdentifier::new(&event_source.transaction_hash, chain_id),
                evm_to_icp_fee,
                nat_to_ledger_mint_index(&mint_block_index),
            ),
            AppicEventPayload::QuarantinedDeposit { event_source } => s
                .record_quarantined_evm_to_icp(EvmToIcpTxIdentifier::new(
                    &event_source.transaction_hash,
                    chain_id,
                )),
            AppicEventPayload::QuarantinedReimbursement { index } => s
                .record_quarantined_reimbursed_icp_to_evm(IcpToEvmIdentifier::new(
                    index.into(),
                    chain_id,
                )),
        });
    }
}
