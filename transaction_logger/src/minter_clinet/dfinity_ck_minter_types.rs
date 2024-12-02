use crate::minter_clinet::appic_minter_types::events::{
    TransactionReceipt as AppicTransactionReceipt, TransactionStatus as AppicTransactionStatus,
    UnsignedTransaction as AppicUnsignedTransaction,
};

use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;
#[derive(CandidType, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum CandidBlockTag {
    Latest,
    Safe,
    Finalized,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, CandidType, Deserialize)]
pub enum EthereumNetwork {
    Mainnet,
    Sepolia,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct InitArg {
    pub ethereum_network: EthereumNetwork,
    pub ecdsa_key_name: String,
    pub ethereum_contract_address: Option<String>,
    pub ledger_id: Principal,
    pub ethereum_block_height: CandidBlockTag,
    pub minimum_withdrawal_amount: Nat,
    pub next_transaction_nonce: Nat,
    pub last_scraped_block_number: Nat,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, CandidType, Deserialize)]
pub struct UpgradeArg {
    pub next_transaction_nonce: Option<Nat>,
    pub minimum_withdrawal_amount: Option<Nat>,
    pub ethereum_contract_address: Option<String>,
    pub ethereum_block_height: Option<CandidBlockTag>,
    pub ledger_suite_orchestrator_id: Option<Principal>,
    pub erc20_helper_contract_address: Option<String>,
    pub last_erc20_scraped_block_number: Option<Nat>,
    pub evm_rpc_id: Option<Principal>,
    pub deposit_with_subaccount_helper_contract_address: Option<String>,
    pub last_deposit_with_subaccount_scraped_block_number: Option<Nat>,
}

pub mod events {

    use super::*;
    use candid::{CandidType, Deserialize, Nat, Principal};
    use serde_bytes::ByteBuf;

    #[derive(Clone, Debug, CandidType, Deserialize)]
    pub struct GetEventsArg {
        pub start: u64,
        pub length: u64,
    }

    #[derive(Clone, Debug, CandidType, Deserialize)]
    pub struct GetEventsResult {
        pub events: Vec<Event>,
        pub total_event_count: u64,
    }

    #[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
    pub struct Event {
        pub timestamp: u64,
        pub payload: EventPayload,
    }

    #[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
    pub struct EventSource {
        pub transaction_hash: String,
        pub log_index: Nat,
    }

    #[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
    pub enum ReimbursementIndex {
        CkEth {
            ledger_burn_index: Nat,
        },
        CkErc20 {
            cketh_ledger_burn_index: Nat,
            ledger_id: Principal,
            ckerc20_ledger_burn_index: Nat,
        },
    }

    #[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
    pub struct AccessListItem {
        pub address: String,
        pub storage_keys: Vec<ByteBuf>,
    }

    #[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
    pub struct UnsignedTransaction {
        pub chain_id: Nat,
        pub nonce: Nat,
        pub max_priority_fee_per_gas: Nat,
        pub max_fee_per_gas: Nat,
        pub gas_limit: Nat,
        pub destination: String,
        pub value: Nat,
        pub data: ByteBuf,
        pub access_list: Vec<AccessListItem>,
    }

    impl From<UnsignedTransaction> for AppicUnsignedTransaction {
        fn from(value: UnsignedTransaction) -> Self {
            Self {
                chain_id: value.chain_id,
                nonce: value.nonce,
                max_priority_fee_per_gas: value.max_priority_fee_per_gas,
                max_fee_per_gas: value.max_fee_per_gas,
                gas_limit: value.gas_limit,
                destination: value.destination,
                value: value.value,
                data: value.data,
                access_list: vec![],
            }
        }
    }

    #[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
    pub enum TransactionStatus {
        Success,
        Failure,
    }

    impl From<TransactionStatus> for AppicTransactionStatus {
        fn from(value: TransactionStatus) -> Self {
            match value {
                TransactionStatus::Success => Self::Success,
                TransactionStatus::Failure => Self::Failure,
            }
        }
    }

    #[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
    pub struct TransactionReceipt {
        pub block_hash: String,
        pub block_number: Nat,
        pub effective_gas_price: Nat,
        pub gas_used: Nat,
        pub status: TransactionStatus,
        pub transaction_hash: String,
    }

    impl From<TransactionReceipt> for AppicTransactionReceipt {
        fn from(value: TransactionReceipt) -> Self {
            Self {
                block_hash: value.block_hash,
                block_number: value.block_number,
                effective_gas_price: value.effective_gas_price,
                gas_used: value.gas_used,
                status: value.status.into(),
                transaction_hash: value.transaction_hash,
            }
        }
    }

    #[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
    pub enum EventPayload {
        Init(InitArg),
        Upgrade(UpgradeArg),
        AcceptedDeposit {
            transaction_hash: String,
            block_number: Nat,
            log_index: Nat,
            from_address: String,
            value: Nat,
            principal: Principal,
            subaccount: Option<[u8; 32]>,
        },
        AcceptedErc20Deposit {
            transaction_hash: String,
            block_number: Nat,
            log_index: Nat,
            from_address: String,
            value: Nat,
            principal: Principal,
            erc20_contract_address: String,
            subaccount: Option<[u8; 32]>,
        },
        InvalidDeposit {
            event_source: EventSource,
            reason: String,
        },
        MintedCkEth {
            event_source: EventSource,
            mint_block_index: Nat,
        },
        SyncedToBlock {
            block_number: Nat,
        },
        SyncedErc20ToBlock {
            block_number: Nat,
        },
        SyncedDepositWithSubaccountToBlock {
            block_number: Nat,
        },
        AcceptedEthWithdrawalRequest {
            withdrawal_amount: Nat,
            destination: String,
            ledger_burn_index: Nat,
            from: Principal,
            from_subaccount: Option<[u8; 32]>,
            created_at: Option<u64>,
        },
        CreatedTransaction {
            withdrawal_id: Nat,
            transaction: UnsignedTransaction,
        },
        SignedTransaction {
            withdrawal_id: Nat,
            raw_transaction: String,
        },
        ReplacedTransaction {
            withdrawal_id: Nat,
            transaction: UnsignedTransaction,
        },
        FinalizedTransaction {
            withdrawal_id: Nat,
            transaction_receipt: TransactionReceipt,
        },
        ReimbursedEthWithdrawal {
            reimbursed_in_block: Nat,
            withdrawal_id: Nat,
            reimbursed_amount: Nat,
            transaction_hash: Option<String>,
        },
        ReimbursedErc20Withdrawal {
            withdrawal_id: Nat,
            burn_in_block: Nat,
            reimbursed_in_block: Nat,
            ledger_id: Principal,
            reimbursed_amount: Nat,
            transaction_hash: Option<String>,
        },
        SkippedBlock {
            contract_address: Option<String>,
            block_number: Nat,
        },
        AddedCkErc20Token {
            chain_id: Nat,
            address: String,
            ckerc20_token_symbol: String,
            ckerc20_ledger_id: Principal,
        },
        AcceptedErc20WithdrawalRequest {
            max_transaction_fee: Nat,
            withdrawal_amount: Nat,
            erc20_contract_address: String,
            destination: String,
            cketh_ledger_burn_index: Nat,
            ckerc20_ledger_id: Principal,
            ckerc20_ledger_burn_index: Nat,
            from: Principal,
            from_subaccount: Option<[u8; 32]>,
            created_at: u64,
        },
        FailedErc20WithdrawalRequest {
            withdrawal_id: Nat,
            reimbursed_amount: Nat,
            to: Principal,
            to_subaccount: Option<[u8; 32]>,
        },
        MintedCkErc20 {
            event_source: EventSource,
            mint_block_index: Nat,
            ckerc20_token_symbol: String,
            erc20_contract_address: String,
        },
        QuarantinedDeposit {
            event_source: EventSource,
        },
        QuarantinedReimbursement {
            index: ReimbursementIndex,
        },
    }
}
