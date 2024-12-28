use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;

#[derive(CandidType, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CandidBlockTag {
    Latest,
    Safe,
    Finalized,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InitArg {
    pub evm_network: EvmNetwork,
    pub ecdsa_key_name: String,
    pub helper_contract_address: Option<String>,
    pub native_ledger_id: Principal,
    pub native_index_id: Principal,
    pub native_symbol: String,
    pub block_height: CandidBlockTag,
    pub native_minimum_withdrawal_amount: Nat,
    pub native_ledger_transfer_fee: Nat,
    pub next_transaction_nonce: Nat,
    pub last_scraped_block_number: Nat,
    pub min_max_priority_fee_per_gas: Nat,
    pub ledger_suite_manager_id: Principal,
}
#[derive(
    CandidType, Clone, Copy, Deserialize, Debug, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord,
)]
pub enum EvmNetwork {
    Ethereum,
    Sepolia,
    ArbitrumOne,
    BSC,
    BSCTestnet,
    Polygon,
    Optimism,
    Base,
    Avalanche,
    Fantom,
}

impl EvmNetwork {
    pub fn chain_id(&self) -> u64 {
        match self {
            EvmNetwork::Ethereum => 1,
            EvmNetwork::Sepolia => 11155111,
            EvmNetwork::ArbitrumOne => 42161,
            EvmNetwork::BSC => 56,
            EvmNetwork::Polygon => 137,
            EvmNetwork::Optimism => 10,
            EvmNetwork::Base => 8453,
            EvmNetwork::Avalanche => 43114,
            EvmNetwork::Fantom => 250,
            EvmNetwork::BSCTestnet => 97,
        }
    }
}

impl TryFrom<u64> for EvmNetwork {
    type Error = String;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(EvmNetwork::Ethereum),
            11155111 => Ok(EvmNetwork::Sepolia),
            42161 => Ok(EvmNetwork::ArbitrumOne),
            56 => Ok(EvmNetwork::BSC),
            137 => Ok(EvmNetwork::Polygon),
            10 => Ok(EvmNetwork::Optimism),
            8453 => Ok(EvmNetwork::Base),
            43114 => Ok(EvmNetwork::Avalanche),
            250 => Ok(EvmNetwork::Fantom),
            97 => Ok(EvmNetwork::BSCTestnet),
            _ => Err("Unknown EVM chain id Network".to_string()),
        }
    }
}

#[derive(
    CandidType, Deserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize,
)]
pub struct UpgradeArg {
    pub next_transaction_nonce: Option<Nat>,
    pub native_minimum_withdrawal_amount: Option<Nat>,
    pub helper_contract_address: Option<String>,
    pub block_height: Option<CandidBlockTag>,
    pub last_scraped_block_number: Option<Nat>,
    pub evm_rpc_id: Option<Principal>,
    pub native_ledger_transfer_fee: Option<Nat>,
    pub min_max_priority_fee_per_gas: Option<Nat>,
}

pub mod events {

    use super::*;
    use crate::{numeric::LedgerBurnIndex, state::nat_to_ledger_burn_index};
    use candid::{CandidType, Deserialize, Nat, Principal};
    use serde_bytes::ByteBuf;

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub struct GetEventsArg {
        pub start: u64,
        pub length: u64,
    }

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub struct GetEventsResult {
        pub events: Vec<Event>,
        pub total_event_count: u64,
    }

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub struct Event {
        pub timestamp: u64,
        pub payload: EventPayload,
    }

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub struct EventSource {
        pub transaction_hash: String,
        pub log_index: Nat,
    }

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub enum ReimbursementIndex {
        Native {
            ledger_burn_index: Nat,
        },
        Erc20 {
            native_ledger_burn_index: Nat,
            ledger_id: Principal,
            erc20_ledger_burn_index: Nat,
        },
    }

    impl From<ReimbursementIndex> for LedgerBurnIndex {
        fn from(value: ReimbursementIndex) -> Self {
            match value {
                ReimbursementIndex::Native { ledger_burn_index } => {
                    nat_to_ledger_burn_index(&ledger_burn_index)
                }
                ReimbursementIndex::Erc20 {
                    native_ledger_burn_index,
                    ledger_id: _,
                    erc20_ledger_burn_index: _,
                } => nat_to_ledger_burn_index(&native_ledger_burn_index),
            }
        }
    }

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub struct AccessListItem {
        pub address: String,
        pub storage_keys: Vec<ByteBuf>,
    }

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
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

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub enum TransactionStatus {
        Success,
        Failure,
    }

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub struct TransactionReceipt {
        pub block_hash: String,
        pub block_number: Nat,
        pub effective_gas_price: Nat,
        pub gas_used: Nat,
        pub status: TransactionStatus,
        pub transaction_hash: String,
    }

    #[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
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
        MintedNative {
            event_source: EventSource,
            mint_block_index: Nat,
        },
        SyncedToBlock {
            block_number: Nat,
        },

        AcceptedNativeWithdrawalRequest {
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
        ReimbursedNativeWithdrawal {
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
            block_number: Nat,
        },
        AddedErc20Token {
            chain_id: Nat,
            address: String,
            erc20_token_symbol: String,
            erc20_ledger_id: Principal,
        },
        AcceptedErc20WithdrawalRequest {
            max_transaction_fee: Nat,
            withdrawal_amount: Nat,
            erc20_contract_address: String,
            destination: String,
            native_ledger_burn_index: Nat,
            erc20_ledger_id: Principal,
            erc20_ledger_burn_index: Nat,
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
        MintedErc20 {
            event_source: EventSource,
            mint_block_index: Nat,
            erc20_token_symbol: String,
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
