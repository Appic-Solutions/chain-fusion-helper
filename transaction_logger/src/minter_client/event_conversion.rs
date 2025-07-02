use crate::minter_client::appic_minter_types::events::Event as AppicEvent;

use crate::minter_client::appic_minter_types::events::EventPayload as AppicEventPayload;
use crate::minter_client::dfinity_ck_minter_types::events::EventPayload as DfinityEventPayload;

use crate::minter_client::{AppicGetEventsResult, DfinityCkGetEventsResult};

use super::appic_minter_types::events::EventSource as AppicEventSource;

// standard type for events returned from minters
#[derive(PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct Events {
    pub events: Vec<AppicEvent>,
}

// A trait for filtering and mapping EventResults form both appic and dfinity cketh minters into a Standard Event type
pub trait Reduce {
    fn reduce(self) -> Events;
}

impl Reduce for DfinityCkGetEventsResult {
    fn reduce(self) -> Events {
        Events {
            events: AppicGetEventsResult::from(self).events,
        }
    }
}

impl Reduce for AppicGetEventsResult {
    fn reduce(self) -> Events {
        let reduced: Vec<AppicEvent> = self
            .events
            .into_iter()
            .filter(|event| {
                matches!(
                    event.payload,
                    AppicEventPayload::Init(..)
                        | AppicEventPayload::Upgrade(..)
                        | AppicEventPayload::AcceptedDeposit { .. }
                        | AppicEventPayload::AcceptedErc20Deposit { .. }
                        | AppicEventPayload::MintedNative { .. }
                        | AppicEventPayload::MintedErc20 { .. }
                        | AppicEventPayload::AcceptedNativeWithdrawalRequest { .. }
                        | AppicEventPayload::CreatedTransaction { .. }
                        | AppicEventPayload::SignedTransaction { .. }
                        | AppicEventPayload::ReplacedTransaction { .. }
                        | AppicEventPayload::FinalizedTransaction { .. }
                        | AppicEventPayload::ReimbursedNativeWithdrawal { .. }
                        | AppicEventPayload::ReimbursedErc20Withdrawal { .. }
                        | AppicEventPayload::AcceptedErc20WithdrawalRequest { .. }
                        | AppicEventPayload::FailedErc20WithdrawalRequest { .. }
                        | AppicEventPayload::InvalidDeposit { .. }
                        | AppicEventPayload::QuarantinedDeposit { .. }
                        | AppicEventPayload::QuarantinedReimbursement { .. }
                        | AppicEventPayload::DeployedWrappedIcrcToken { .. }
                        | AppicEventPayload::AcceptedWrappedIcrcBurn { .. }
                        | AppicEventPayload::QuarantinedRelease { .. }
                        | AppicEventPayload::FailedIcrcLockRequest { .. }
                        | AppicEventPayload::ReleasedIcrcToken { .. }
                        | AppicEventPayload::ReimbursedIcrcWrap { .. }
                )
            })
            .collect();
        Events { events: reduced }
    }
}

impl From<DfinityCkGetEventsResult> for AppicGetEventsResult {
    fn from(value: DfinityCkGetEventsResult) -> AppicGetEventsResult {
        let filtered_mapped: Vec<AppicEvent> = value
            .events
            .into_iter()
            .filter_map(|event| {
                let timestamp = event.timestamp;

                let event_payload = match event.payload {
                    DfinityEventPayload::Init(..)
                    | DfinityEventPayload::Upgrade(..)
                    | DfinityEventPayload::SyncedToBlock { .. }
                    | DfinityEventPayload::SyncedErc20ToBlock { .. }
                    | DfinityEventPayload::SyncedDepositWithSubaccountToBlock { .. }
                    | DfinityEventPayload::SkippedBlock { .. }
                    | DfinityEventPayload::AddedCkErc20Token { .. } => None,

                    DfinityEventPayload::AcceptedDeposit {
                        transaction_hash,
                        block_number,
                        log_index,
                        from_address,
                        value,
                        principal,
                        subaccount,
                    } => Some(AppicEventPayload::AcceptedDeposit {
                        transaction_hash,
                        block_number,
                        log_index,
                        from_address,
                        value,
                        principal,
                        subaccount,
                    }),

                    DfinityEventPayload::AcceptedErc20Deposit {
                        transaction_hash,
                        block_number,
                        log_index,
                        from_address,
                        value,
                        principal,
                        erc20_contract_address,
                        subaccount,
                    } => Some(AppicEventPayload::AcceptedErc20Deposit {
                        transaction_hash,
                        block_number,
                        log_index,
                        from_address,
                        value,
                        principal,
                        erc20_contract_address,
                        subaccount,
                    }),

                    DfinityEventPayload::InvalidDeposit {
                        event_source,
                        reason,
                    } => Some(AppicEventPayload::InvalidDeposit {
                        event_source: AppicEventSource {
                            log_index: event_source.log_index,
                            transaction_hash: event_source.transaction_hash,
                        },
                        reason,
                    }),

                    DfinityEventPayload::MintedCkEth {
                        event_source,
                        mint_block_index,
                    } => Some(AppicEventPayload::MintedNative {
                        event_source: AppicEventSource {
                            log_index: event_source.log_index,
                            transaction_hash: event_source.transaction_hash,
                        },
                        mint_block_index,
                    }),

                    DfinityEventPayload::AcceptedEthWithdrawalRequest {
                        withdrawal_amount,
                        destination,
                        ledger_burn_index,
                        from,
                        from_subaccount,
                        created_at,
                    } => Some(AppicEventPayload::AcceptedNativeWithdrawalRequest {
                        withdrawal_amount,
                        destination,
                        ledger_burn_index,
                        from,
                        from_subaccount,
                        created_at,
                        l1_fee: None,
                        withdrawal_fee: None,
                    }),

                    DfinityEventPayload::CreatedTransaction {
                        withdrawal_id,
                        transaction,
                    } => Some(AppicEventPayload::CreatedTransaction {
                        withdrawal_id,
                        transaction: transaction.into(),
                    }),

                    DfinityEventPayload::SignedTransaction {
                        withdrawal_id,
                        raw_transaction,
                    } => Some(AppicEventPayload::SignedTransaction {
                        withdrawal_id,
                        raw_transaction,
                    }),

                    DfinityEventPayload::ReplacedTransaction {
                        withdrawal_id,
                        transaction,
                    } => Some(AppicEventPayload::ReplacedTransaction {
                        withdrawal_id,
                        transaction: transaction.into(),
                    }),

                    DfinityEventPayload::FinalizedTransaction {
                        withdrawal_id,
                        transaction_receipt,
                    } => Some(AppicEventPayload::FinalizedTransaction {
                        withdrawal_id,
                        transaction_receipt: transaction_receipt.into(),
                    }),

                    DfinityEventPayload::ReimbursedEthWithdrawal {
                        reimbursed_in_block,
                        withdrawal_id,
                        reimbursed_amount,
                        transaction_hash,
                    } => Some(AppicEventPayload::ReimbursedNativeWithdrawal {
                        reimbursed_in_block,
                        withdrawal_id,
                        reimbursed_amount,
                        transaction_hash,
                    }),

                    DfinityEventPayload::ReimbursedErc20Withdrawal {
                        withdrawal_id,
                        burn_in_block,
                        reimbursed_in_block,
                        ledger_id,
                        reimbursed_amount,
                        transaction_hash,
                    } => Some(AppicEventPayload::ReimbursedErc20Withdrawal {
                        withdrawal_id,
                        burn_in_block,
                        reimbursed_in_block,
                        ledger_id,
                        reimbursed_amount,
                        transaction_hash,
                    }),

                    DfinityEventPayload::AcceptedErc20WithdrawalRequest {
                        max_transaction_fee,
                        withdrawal_amount,
                        erc20_contract_address,
                        destination,
                        cketh_ledger_burn_index,
                        ckerc20_ledger_id,
                        ckerc20_ledger_burn_index,
                        from,
                        from_subaccount,
                        created_at,
                    } => Some(AppicEventPayload::AcceptedErc20WithdrawalRequest {
                        max_transaction_fee,
                        withdrawal_amount,
                        erc20_contract_address,
                        destination,
                        native_ledger_burn_index: cketh_ledger_burn_index,
                        erc20_ledger_id: ckerc20_ledger_id,
                        erc20_ledger_burn_index: ckerc20_ledger_burn_index,
                        from,
                        from_subaccount,
                        created_at,
                        l1_fee: None,
                        withdrawal_fee: None,
                        is_wrapped_mint: false,
                    }),

                    DfinityEventPayload::MintedCkErc20 {
                        event_source,
                        mint_block_index,
                        ckerc20_token_symbol,
                        erc20_contract_address,
                    } => Some(AppicEventPayload::MintedErc20 {
                        event_source: AppicEventSource {
                            log_index: event_source.log_index,
                            transaction_hash: event_source.transaction_hash,
                        },
                        mint_block_index,
                        erc20_token_symbol: ckerc20_token_symbol,
                        erc20_contract_address,
                    }),

                    DfinityEventPayload::QuarantinedDeposit { event_source } => {
                        Some(AppicEventPayload::QuarantinedDeposit {
                            event_source: AppicEventSource {
                                log_index: event_source.log_index,
                                transaction_hash: event_source.transaction_hash,
                            },
                        })
                    }

                    DfinityEventPayload::QuarantinedReimbursement { index } => {
                        Some(AppicEventPayload::QuarantinedReimbursement {
                            index: index.into(),
                        })
                    }
                    DfinityEventPayload::FailedErc20WithdrawalRequest {
                        withdrawal_id,
                        reimbursed_amount,
                        to,
                        to_subaccount,
                    } => Some(AppicEventPayload::FailedErc20WithdrawalRequest {
                        withdrawal_id,
                        reimbursed_amount,
                        to,
                        to_subaccount,
                    }),
                };

                event_payload.map(|payload| AppicEvent { timestamp, payload })
            })
            .collect();

        AppicGetEventsResult {
            events: filtered_mapped,
            total_event_count: value.total_event_count,
        }
    }
}
