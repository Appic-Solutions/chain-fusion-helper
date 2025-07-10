use std::str::FromStr;

use crate::{
    guard::TimerGuard,
    logs::{DEBUG, INFO},
    minter_client::{
        appic_minter_types::{InitArg, UpgradeArg},
        MinterClient,
    },
    state::{
        mutate_state, nat_to_erc20_amount, nat_to_ledger_burn_index, nat_to_ledger_mint_index,
        read_state,
        types::{
            ChainId, Erc20Identifier, EvmToIcpTxIdentifier, IcpToEvmIdentifier, MinterKey, Operator,
        },
    },
};

use crate::address::Address;
use crate::minter_client::appic_minter_types::events::EventPayload as AppicEventPayload;
use ic_canister_log::log;

use crate::minter_client::event_conversion::Events;
const MAX_EVENTS_PER_RESPONSE: u64 = 100;

pub const NATIVE_ERC20_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

pub async fn scrape_events() {
    // Issue a timer guard
    let _guard = match TimerGuard::new(crate::guard::TaskType::ScrapeEvents) {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let minters = read_state(|s| s.get_active_minters());

    // Scrape only active minters
    for (minter_key, minter) in minters.iter() {
        let minter_client = MinterClient::from(minter);

        // Get the latest event count to update last_observed_event;
        // -1 since the starting index in 0 not 1
        let latest_event_count = minter_client.get_total_events_count().await - 1;

        // Check if the previous last_observed_event is greater or equal to latest one;
        // If yes there should be no scraping for events and last_observed_event should not be updated
        if minter.last_observed_event >= latest_event_count {
            continue;
        };

        // Updating last observed event count
        mutate_state(|s| s.update_last_observed_event(minter_key, latest_event_count));

        let last_scraped_event = minter.last_scraped_event;

        // Scraping logs between specified ranges
        // MAX_EVENT_RESPONSE= 100 so the log range should not be more than 100
        // min((last_observed_event - last_scraped_event),100) will be the specified range
        // If last_observed_event - last_scraped_event contains more than 100, the event scaping will be divided into multiple calls

        scrape_events_range(
            latest_event_count,
            last_scraped_event,
            MAX_EVENTS_PER_RESPONSE,
            &minter_client,
            minter_key,
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
) {
    if last_scraped_event >= last_observed_event {
        log!(
            INFO,
            "[Scraping Events] No events to scrape. All events are already processed."
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
                    log!(INFO, "[Scraping Events] Received Event {:?}", events);

                    apply_state_transition(events, minter_key.operator(), minter_key.chain_id());
                    mutate_state(|s| s.update_last_scraped_event(minter_key, chunk_end));
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

fn apply_state_transition(events: Events, operator: Operator, chain_id: ChainId) {
    for event in events.events.into_iter() {
        // Applying the state transition
        mutate_state(|s| match event.payload {
            AppicEventPayload::Init(InitArg {
                evm_network: _,
                ecdsa_key_name: _,
                helper_contract_address: _,
                native_ledger_id,
                native_index_id: _,
                native_symbol,
                block_height: _,
                native_minimum_withdrawal_amount: _,
                native_ledger_transfer_fee,
                next_transaction_nonce: _,
                last_scraped_block_number: _,
                min_max_priority_fee_per_gas: _,
                ledger_suite_manager_id: _,
                deposit_native_fee: _,
                withdrawal_native_fee,
            }) => {
                s.update_minter_fees(
                    &MinterKey(chain_id, operator),
                    nat_to_erc20_amount(withdrawal_native_fee),
                );

                s.record_native_icrc_ledger(
                    native_ledger_id,
                    native_symbol,
                    nat_to_erc20_amount(native_ledger_transfer_fee),
                    chain_id,
                );
            }
            AppicEventPayload::Upgrade(UpgradeArg {
                next_transaction_nonce: _,
                native_minimum_withdrawal_amount: _,
                helper_contract_address: _,
                block_height: _,
                last_scraped_block_number: _,
                evm_rpc_id: _,
                native_ledger_transfer_fee: _,
                min_max_priority_fee_per_gas: _,
                deposit_native_fee: _,
                withdrawal_native_fee,
            }) => {
                if let Some(new_fee) = withdrawal_native_fee {
                    s.update_minter_fees(
                        &MinterKey(chain_id, operator),
                        nat_to_erc20_amount(new_fee),
                    );
                }
            }
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
                nat_to_ledger_mint_index(&mint_block_index),
                None,
            ),
            AppicEventPayload::SyncedToBlock { .. } => {}
            AppicEventPayload::AcceptedNativeWithdrawalRequest {
                withdrawal_amount,
                destination,
                ledger_burn_index,
                from,
                from_subaccount,
                created_at,
                l1_fee,
                withdrawal_fee,
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
                l1_fee,
                withdrawal_fee,
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
                erc20_ledger_id: _,
                l1_fee,
                withdrawal_fee,
                is_wrapped_mint: _,
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
                l1_fee,
                withdrawal_fee,
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
                nat_to_ledger_mint_index(&mint_block_index),
                None,
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
            AppicEventPayload::AcceptedWrappedIcrcBurn {
                transaction_hash,
                block_number,
                log_index: _,
                from_address,
                value,
                principal,
                wrapped_erc20_contract_address,
                icrc_token_principal: _,
                subaccount,
            } => s.record_accepted_evm_to_icp(
                EvmToIcpTxIdentifier::new(&transaction_hash, chain_id),
                transaction_hash,
                block_number,
                from_address,
                value,
                principal,
                wrapped_erc20_contract_address,
                subaccount,
                chain_id,
                operator,
                event.timestamp,
            ),
            AppicEventPayload::InvalidEvent {
                event_source,
                reason,
            } => s.record_invalid_evm_to_icp(
                EvmToIcpTxIdentifier::new(&event_source.transaction_hash, chain_id),
                reason,
            ),
            AppicEventPayload::DeployedWrappedIcrcToken {
                transaction_hash: _,
                block_number: _,
                log_index: _,
                base_token,
                deployed_wrapped_erc20,
            } => s.record_deployed_wrapped_icrc_token(
                base_token,
                Erc20Identifier::new(
                    &Address::from_str(&deployed_wrapped_erc20).unwrap(),
                    chain_id,
                ),
            ),
            AppicEventPayload::QuarantinedRelease { event_source } => s
                .record_quarantined_evm_to_icp(EvmToIcpTxIdentifier::new(
                    &event_source.transaction_hash,
                    chain_id,
                )),
            AppicEventPayload::ReleasedIcrcToken {
                event_source,
                release_block_index,
                transfer_fee,
            } => s.record_minted_evm_to_icp(
                EvmToIcpTxIdentifier::new(&event_source.transaction_hash, chain_id),
                nat_to_ledger_burn_index(&release_block_index),
                Some(transfer_fee),
            ),
            AppicEventPayload::FailedIcrcLockRequest {
                withdrawal_id,
                reimbursed_amount: _,
                to: _,
                to_subaccount: _,
            } => s.record_reimbursed_icp_to_evm(IcpToEvmIdentifier::new(
                nat_to_ledger_burn_index(&withdrawal_id),
                chain_id,
            )),
            AppicEventPayload::ReimbursedIcrcWrap {
                native_ledger_burn_index,
                ..
            } => s.record_reimbursed_icp_to_evm(IcpToEvmIdentifier::new(
                nat_to_ledger_burn_index(&native_ledger_burn_index),
                chain_id,
            )),
        });
    }
}
