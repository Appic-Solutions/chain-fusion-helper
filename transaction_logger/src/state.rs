use candid::{CandidType, Nat, Principal};
use ic_ethereum_types::Address;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{storable::Bound, BTreeMap, Storable};
use minicbor::{Decode, Encode};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use storage_config::{
    evm_to_icp_memory, icp_to_evm_memory, minter_memory, supported_appic_tokens_memory_id,
    supported_ckerc20_tokens_memory_id,
};

use std::str::FromStr;

use crate::endpoints::{
    AddEvmToIcpTx, AddIcpToEvmTx, CandidEvmToIcp, CandidIcpToEvm, MinterArgs, TokenPair,
    Transaction,
};
use crate::scrape_events::NATIVE_ERC20_ADDRESS;

use std::fmt::Debug;

use crate::minter_clinet::appic_minter_types::events::{TransactionReceipt, TransactionStatus};

#[derive(
    Clone,
    Copy,
    CandidType,
    PartialEq,
    Encode,
    Decode,
    PartialOrd,
    Eq,
    Ord,
    Debug,
    Deserialize,
    Serialize,
)]
pub enum Oprator {
    #[n(0)]
    DfinityCkEthMinter,
    #[n(1)]
    AppicMinter,
}

#[derive(Clone, PartialEq, Ord, PartialOrd, Eq, Debug, Encode, Decode, Deserialize, Serialize)]
pub struct Minter {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub id: Principal,
    #[n(1)]
    pub last_observed_event: u64,
    #[n(2)]
    pub last_scraped_event: u64,
    #[n(3)]
    pub oprator: Oprator,
    #[cbor(n(4), with = "crate::cbor::nat")]
    pub evm_to_icp_fee: Nat,
    #[cbor(n(5), with = "crate::cbor::nat")]
    pub icp_to_evm_fee: Nat,
    #[n(6)]
    pub chain_id: ChainId,
}

impl Minter {
    pub fn update_last_observed_event(&mut self, event: u64) {
        self.last_observed_event = event
    }

    pub fn update_last_scraped_event(&mut self, event: u64) {
        self.last_scraped_event = event
    }

    pub fn from_minter_args(args: MinterArgs) -> Self {
        let MinterArgs {
            chain_id,
            minter_id,
            oprator,
            last_observed_event,
            last_scraped_event,
            evm_to_icp_fee,
            icp_to_evm_fee,
        } = args;
        Self {
            id: minter_id,
            last_observed_event: nat_to_u64(&last_observed_event),
            last_scraped_event: nat_to_u64(&last_scraped_event),
            oprator,
            evm_to_icp_fee,
            icp_to_evm_fee,
            chain_id: ChainId::from(&chain_id),
        }
    }
}

#[derive(Clone, PartialEq, Encode, Decode, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct MinterKey(#[n(0)] pub ChainId, #[n(1)] pub Oprator);

impl MinterKey {
    pub fn oprator(&self) -> Oprator {
        self.1
    }

    pub fn chain_id(&self) -> ChainId {
        self.0
    }
}

impl From<&Minter> for MinterKey {
    fn from(value: &Minter) -> Self {
        Self(value.chain_id, value.oprator)
    }
}

type TransactionHash = String;

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Encode, Decode, Deserialize, Serialize)]
pub struct EvmToIcpTxIdentifier(#[n(0)] TransactionHash, #[n(1)] ChainId);

impl EvmToIcpTxIdentifier {
    pub fn new(transaction_hash: &TransactionHash, chain_id: ChainId) -> Self {
        EvmToIcpTxIdentifier(transaction_hash.clone(), chain_id)
    }
}
impl From<&AddEvmToIcpTx> for EvmToIcpTxIdentifier {
    fn from(value: &AddEvmToIcpTx) -> Self {
        Self::new(&value.transaction_hash, ChainId::from(&value.chain_id))
    }
}

#[derive(
    Clone, Encode, Decode, CandidType, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize,
)]
pub enum EvmToIcpStatus {
    #[n(0)]
    PendingVerification,
    #[n(1)]
    Accepted,
    #[n(2)]
    Minted,
    #[n(3)]
    Invalid(#[n(0)] String),
    #[n(4)]
    Quarantined,
}

#[derive(Clone, Encode, Decode, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct EvmToIcpTx {
    #[n(0)]
    pub from_address: Address,
    #[n(1)]
    pub transaction_hash: TransactionHash,
    #[cbor(n(2), with = "crate::cbor::nat")]
    pub value: Nat,
    #[cbor(n(3), with = "crate::cbor::nat::option")]
    pub block_number: Option<Nat>,
    #[cbor(n(4), with = "crate::cbor::nat::option")]
    pub actual_received: Option<Nat>,
    #[cbor(n(5), with = "crate::cbor::principal")]
    pub principal: Principal,
    #[cbor(n(6), with = "minicbor::bytes")]
    pub subaccount: Option<[u8; 32]>,
    #[n(7)]
    pub chain_id: ChainId,
    #[cbor(n(8), with = "crate::cbor::nat::option")]
    pub total_gas_spent: Option<Nat>,
    #[n(9)]
    pub erc20_contract_address: Address,
    #[cbor(n(10), with = "crate::cbor::principal::option")]
    pub icrc_ledger_id: Option<Principal>,
    #[n(11)]
    pub status: EvmToIcpStatus,
    #[n(12)]
    pub verified: bool,
    #[n(13)]
    pub time: u64,
    #[n(14)]
    pub oprator: Oprator,
}

pub type NativeLedgerBurnIndex = Nat;

#[derive(Clone, Encode, Decode, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct IcpToEvmIdentifier(
    #[cbor(n(0), with = "crate::cbor::nat")] NativeLedgerBurnIndex,
    #[n(1)] ChainId,
);
impl IcpToEvmIdentifier {
    pub fn new(native_ledger_burn_index: &NativeLedgerBurnIndex, chain_id: ChainId) -> Self {
        IcpToEvmIdentifier(native_ledger_burn_index.clone(), chain_id)
    }
}

impl From<&AddIcpToEvmTx> for IcpToEvmIdentifier {
    fn from(value: &AddIcpToEvmTx) -> Self {
        Self::new(
            &value.native_ledger_burn_index,
            ChainId::from(&value.chain_id),
        )
    }
}

#[derive(
    CandidType, Clone, Encode, Decode, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize,
)]
pub enum IcpToEvmStatus {
    #[n(0)]
    PendingVerification,
    #[n(1)]
    Accepted,
    #[n(2)]
    Created,
    #[n(3)]
    SignedTransaction,
    #[n(4)]
    FinalizedTransaction,
    #[n(5)]
    ReplacedTransaction,
    #[n(6)]
    Reimbursed,
    #[n(7)]
    QuarantinedReimbursement,
    #[n(8)]
    Successful,
    #[n(9)]
    Failed,
}

#[derive(Clone, PartialEq, Encode, Decode, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct IcpToEvmTx {
    #[n(0)]
    pub transaction_hash: Option<TransactionHash>,
    #[cbor(n(1), with = "crate::cbor::nat")]
    pub native_ledger_burn_index: NativeLedgerBurnIndex,
    #[cbor(n(2), with = "crate::cbor::nat")]
    pub withdrawal_amount: Nat,
    #[cbor(n(3), with = "crate::cbor::nat::option")]
    pub actual_received: Option<Nat>,
    #[n(4)]
    pub destination: Address,
    #[cbor(n(5), with = "crate::cbor::principal")]
    pub from: Principal,
    #[n(6)]
    pub chain_id: ChainId,
    #[cbor(n(7), with = "minicbor::bytes")]
    pub from_subaccount: Option<[u8; 32]>,
    #[n(8)]
    pub time: u64,
    #[cbor(n(9), with = "crate::cbor::nat::option")]
    pub max_transaction_fee: Option<Nat>,
    #[cbor(n(10), with = "crate::cbor::nat::option")]
    pub effective_gas_price: Option<Nat>,
    #[cbor(n(11), with = "crate::cbor::nat::option")]
    pub gas_used: Option<Nat>,
    #[cbor(n(12), with = "crate::cbor::nat::option")]
    pub toatal_gas_spent: Option<Nat>,
    #[cbor(n(13), with = "crate::cbor::nat::option")]
    pub erc20_ledger_burn_index: Option<Nat>,
    #[n(14)]
    pub erc20_contract_address: Address,
    #[cbor(n(15), with = "crate::cbor::principal::option")]
    pub icrc_ledger_id: Option<Principal>,
    #[n(16)]
    pub verified: bool,
    #[n(17)]
    pub status: IcpToEvmStatus,
    #[n(18)]
    pub oprator: Oprator,
}

#[derive(Clone, PartialEq, Ord, Eq, Encode, Decode, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Erc20Identifier(#[n(0)] pub Address, #[n(1)] pub ChainId);

impl Erc20Identifier {
    pub fn new(contract: &Address, chain_id: ChainId) -> Self {
        Self(contract.clone(), chain_id)
    }

    pub fn erc20_address(&self) -> Address {
        self.0
    }
    pub fn chain_id(&self) -> ChainId {
        self.1
    }
}
// State Definition,
// All types of transactions will be sotred in this stable state
pub struct State {
    // List of all minters including (cketh dfinity and appic minters)
    pub minters: BTreeMap<MinterKey, Minter, StableMemory>,

    // List of all evm_to_icp transactions
    pub evm_to_icp_txs: BTreeMap<EvmToIcpTxIdentifier, EvmToIcpTx, StableMemory>,

    // list of all icp_to_evm transactions
    pub icp_to_evm_txs: BTreeMap<IcpToEvmIdentifier, IcpToEvmTx, StableMemory>,

    pub supported_ckerc20_tokens: BTreeMap<Erc20Identifier, Principal, StableMemory>,
    pub supported_twin_appic_tokens: BTreeMap<Erc20Identifier, Principal, StableMemory>,
}

impl State {
    pub fn update_minter_fees(
        &mut self,
        minter_key: &MinterKey,
        evm_to_icp_fee: Nat,
        icp_to_evm_fee: Nat,
    ) {
        if let Some(minter) = self.minters.get(minter_key) {
            let new_minter = Minter {
                evm_to_icp_fee,
                icp_to_evm_fee,
                ..minter
            };
            self.record_minter(new_minter);
        }
    }

    pub fn update_last_observed_event(&mut self, minter_key: &MinterKey, last_observed_event: u64) {
        if let Some(minter) = self.minters.get(minter_key) {
            let new_minter = Minter {
                last_observed_event,
                ..minter
            };
            self.record_minter(new_minter);
        }
    }

    pub fn update_last_scraped_event(&mut self, minter_key: &MinterKey, last_scraped_event: u64) {
        if let Some(minter) = self.minters.get(minter_key) {
            let new_minter = Minter {
                last_scraped_event,
                ..minter
            };
            self.record_minter(new_minter);
        }
    }

    pub fn get_minters(&self) -> Vec<Minter> {
        self.minters
            .iter()
            .map(|(_minter_key, minter)| minter)
            .collect()
    }

    pub fn if_chain_id_exists(&self, chain_id: ChainId) -> bool {
        for minter in self.get_minters() {
            if minter.chain_id == chain_id {
                return true;
            }
        }
        false
    }

    pub fn record_minter(&mut self, minter: Minter) {
        self.minters.insert(MinterKey::from(&minter), minter);
    }

    pub fn get_icrc_twin_for_erc20(
        &self,
        erc20_identifier: &Erc20Identifier,
        oprator: &Oprator,
    ) -> Option<Principal> {
        match oprator {
            Oprator::AppicMinter => self
                .supported_twin_appic_tokens
                .get(erc20_identifier)
                .map(|token_principal| token_principal),
            Oprator::DfinityCkEthMinter => self
                .supported_ckerc20_tokens
                .get(erc20_identifier)
                .map(|token_principal| token_principal),
        }
    }

    pub fn if_evm_to_icp_tx_exists(&self, identifier: &EvmToIcpTxIdentifier) -> bool {
        self.evm_to_icp_txs.get(identifier).is_some()
    }

    pub fn if_icp_to_evm_tx_exists(&self, identifier: &IcpToEvmIdentifier) -> bool {
        self.icp_to_evm_txs.get(identifier).is_some()
    }

    pub fn record_new_evm_to_icp(&mut self, identifier: EvmToIcpTxIdentifier, tx: EvmToIcpTx) {
        self.evm_to_icp_txs.insert(identifier, tx);
    }

    pub fn record_accepted_evm_to_icp(
        &mut self,
        identifier: EvmToIcpTxIdentifier,
        transaction_hash: TransactionHash,
        block_number: Nat,
        from_address: String,
        value: Nat,
        principal: Principal,
        erc20_contract_address: String,
        subaccount: Option<[u8; 32]>,
        chain_id: ChainId,
        oprator: Oprator,
        timestamp: u64,
    ) {
        // Parse addresses once
        let parsed_from_address = Address::from_str(&from_address)
            .expect("Should not fail converting from_address to Address");
        let parsed_erc20_address = Address::from_str(&erc20_contract_address)
            .expect("Should not fail converting erc20_contract_address to Address");

        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            // Update only the necessary fields in the existing transaction
            let new_tx = EvmToIcpTx {
                verified: true,
                block_number: Some(block_number),
                from_address: parsed_from_address,
                value,
                principal,
                erc20_contract_address: parsed_erc20_address,
                subaccount,
                status: EvmToIcpStatus::Accepted,
                ..tx
            };
            self.record_new_evm_to_icp(identifier, new_tx);
        } else {
            // Create a new transaction only if one doses not already exist
            let new_tx = EvmToIcpTx {
                from_address: parsed_from_address,
                transaction_hash,
                value,
                block_number: Some(block_number),
                actual_received: None,
                principal,
                subaccount,
                chain_id,
                total_gas_spent: None,
                erc20_contract_address: parsed_erc20_address,
                icrc_ledger_id: self.get_icrc_twin_for_erc20(
                    &Erc20Identifier(parsed_erc20_address, chain_id),
                    &oprator,
                ),
                status: EvmToIcpStatus::Accepted,
                verified: true,
                time: timestamp,
                oprator,
            };

            self.record_new_evm_to_icp(identifier, new_tx);
        }
    }

    pub fn record_minted_evm_to_icp(
        &mut self,
        identifier: EvmToIcpTxIdentifier,
        erc20_contract_address: String,
        evm_to_icp_fee: &Nat,
    ) {
        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            // Parse the address once
            let parsed_address = Address::from_str(&erc20_contract_address)
                .expect("Should not fail converting minter address to Address");

            // Fee calculation
            let actual_received = if is_native_token(&parsed_address) {
                Some(tx.value.clone() - evm_to_icp_fee.clone()) // Clone only when needed
            } else {
                Some(tx.value.clone())
            };

            let new_tx = EvmToIcpTx {
                actual_received,
                erc20_contract_address: parsed_address,
                status: EvmToIcpStatus::Minted,
                ..tx
            };
            self.record_new_evm_to_icp(identifier, new_tx);
        }
    }

    pub fn record_invalid_evm_to_icp(&mut self, identifier: EvmToIcpTxIdentifier, reason: String) {
        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            let new_tx = EvmToIcpTx {
                status: EvmToIcpStatus::Invalid(reason),
                ..tx
            };
            self.record_new_evm_to_icp(identifier, new_tx);
        }
    }

    pub fn record_quarantined_evm_to_icp(&mut self, identifier: EvmToIcpTxIdentifier) {
        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            let new_tx = EvmToIcpTx {
                status: EvmToIcpStatus::Quarantined,
                ..tx
            };
            self.record_new_evm_to_icp(identifier, new_tx);
        }
    }

    pub fn record_new_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier, tx: IcpToEvmTx) {
        self.icp_to_evm_txs.insert(identifier, tx);
    }

    pub fn record_accepted_icp_to_evm(
        &mut self,
        identifier: IcpToEvmIdentifier,
        max_transaction_fee: Option<Nat>,
        withdrawal_amount: Nat,
        erc20_contract_address: String,
        destination: String,
        native_ledger_burn_index: Nat,
        erc20_ledger_burn_index: Option<Nat>,
        from: Principal,
        from_subaccount: Option<[u8; 32]>,
        created_at: Option<u64>,
        oprator: Oprator,
        chain_id: ChainId,
        timestamp: u64,
    ) {
        let destination_address = Address::from_str(&destination)
            .expect("Should not fail converting destination to Address");
        let erc20_address = Address::from_str(&erc20_contract_address)
            .expect("Should not fail converting ERC20 contract address to Address");

        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                verified: true,
                max_transaction_fee,
                withdrawal_amount,
                erc20_contract_address: erc20_address,
                destination: destination_address,
                native_ledger_burn_index,
                erc20_ledger_burn_index,
                from,
                from_subaccount,
                status: IcpToEvmStatus::Accepted,
                ..tx
            };

            self.record_new_icp_to_evm(identifier, new_tx);
        } else {
            let icrc_ledger_id = self.get_icrc_twin_for_erc20(
                &Erc20Identifier(erc20_address.clone(), chain_id),
                &oprator,
            );

            let new_tx = IcpToEvmTx {
                native_ledger_burn_index,
                withdrawal_amount,
                actual_received: None,
                destination: destination_address,
                from,
                from_subaccount,
                time: created_at.unwrap_or(timestamp),
                max_transaction_fee,
                erc20_ledger_burn_index,
                icrc_ledger_id,
                chain_id,
                erc20_contract_address: erc20_address,
                verified: true,
                status: IcpToEvmStatus::Accepted,
                oprator,
                effective_gas_price: None,
                gas_used: None,
                toatal_gas_spent: None,
                transaction_hash: None,
            };

            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }
    pub fn record_created_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::Created,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_signed_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::SignedTransaction,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_replaced_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::ReplacedTransaction,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_finalized_icp_to_evm(
        &mut self,
        identifier: IcpToEvmIdentifier,
        receipt: TransactionReceipt,
        icp_to_evm_fee: &Nat,
    ) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let actual_received = if is_native_token(&tx.erc20_contract_address) {
                Some(
                    tx.withdrawal_amount.clone()
                        - (receipt.gas_used.clone() * receipt.effective_gas_price.clone())
                        - icp_to_evm_fee.clone(),
                )
            } else {
                Some(tx.withdrawal_amount.clone())
            };

            let status = match receipt.status {
                TransactionStatus::Success => IcpToEvmStatus::Successful,
                TransactionStatus::Failure => IcpToEvmStatus::Failed,
            };
            let new_tx = IcpToEvmTx {
                actual_received,
                transaction_hash: Some(receipt.transaction_hash),
                gas_used: Some(receipt.gas_used.clone()),
                effective_gas_price: Some(receipt.effective_gas_price.clone()),
                toatal_gas_spent: Some(
                    (receipt.gas_used * receipt.effective_gas_price) + icp_to_evm_fee.clone(),
                ),
                status,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_reimbursed_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::Reimbursed,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_quarantined_reimbursed_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::QuarantinedReimbursement,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn all_unverified_icp_to_evm(&self) -> Vec<(IcpToEvmIdentifier, u64)> {
        self.icp_to_evm_txs
            .iter()
            .filter_map(|(identifier, tx)| {
                if tx.verified == false {
                    Some((identifier.clone(), tx.time))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn remove_unverified_icp_to_evm(&mut self, identifier: &IcpToEvmIdentifier) {
        self.icp_to_evm_txs.remove(identifier);
    }

    pub fn all_unverified_evm_to_icp(&self) -> Vec<(EvmToIcpTxIdentifier, u64)> {
        self.evm_to_icp_txs
            .iter()
            .filter_map(|(identifier, tx)| {
                if tx.verified == false {
                    Some((identifier.clone(), tx.time))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn remove_unverified_evm_to_icp(&mut self, identifier: &EvmToIcpTxIdentifier) {
        self.evm_to_icp_txs.remove(identifier);
    }

    pub fn get_transaction_for_address(&self, address: Address) -> Vec<Transaction> {
        let all_tx: Vec<Transaction> = self
            .evm_to_icp_txs
            .iter()
            .filter_map(|(_id, tx)| {
                if tx.from_address == address {
                    Some(Transaction::from(CandidEvmToIcp::from(tx.clone())))
                } else {
                    None
                }
            })
            .chain(self.icp_to_evm_txs.iter().filter_map(|(_id, tx)| {
                if tx.destination == address {
                    Some(Transaction::from(CandidIcpToEvm::from(tx.clone())))
                } else {
                    None
                }
            }))
            .collect();

        all_tx
    }

    pub fn get_transaction_for_principal(&self, principal_id: Principal) -> Vec<Transaction> {
        let all_tx: Vec<Transaction> = self
            .evm_to_icp_txs
            .iter()
            .filter_map(|(_id, tx)| {
                if tx.principal == principal_id {
                    Some(Transaction::from(CandidEvmToIcp::from(tx.clone())))
                } else {
                    None
                }
            })
            .chain(self.icp_to_evm_txs.iter().filter_map(|(_id, tx)| {
                if tx.from == principal_id {
                    Some(Transaction::from(CandidIcpToEvm::from(tx.clone())))
                } else {
                    None
                }
            }))
            .collect();

        all_tx
    }

    pub fn get_suported_twin_token_pairs(&self) -> Vec<TokenPair> {
        self.supported_ckerc20_tokens
            .iter()
            .map(|(erc20_identifier, ledger_id)| TokenPair {
                erc20_address: erc20_identifier.erc20_address().to_string(),
                ledger_id: ledger_id,
                oprator: Oprator::DfinityCkEthMinter,
                chain_id: erc20_identifier.chain_id().into(),
            })
            .chain(
                self.supported_twin_appic_tokens
                    .iter()
                    .map(|(erc20_identifier, ledger_id)| TokenPair {
                        erc20_address: erc20_identifier.erc20_address().to_string(),
                        ledger_id: ledger_id,
                        oprator: Oprator::AppicMinter,
                        chain_id: erc20_identifier.chain_id().into(),
                    }),
            )
            .collect()
    }
}

pub fn is_native_token(address: &Address) -> bool {
    address
        == &Address::from_str(NATIVE_ERC20_ADDRESS).expect("Should not fail converintg to address")
}

impl From<&Nat> for ChainId {
    fn from(value: &Nat) -> Self {
        Self(value.0.to_u64().unwrap())
    }
}

impl From<ChainId> for Nat {
    fn from(value: ChainId) -> Self {
        Nat::from(value.0)
    }
}

pub fn nat_to_u64(value: &Nat) -> u64 {
    value.0.to_u64().unwrap()
}

#[derive(
    Clone, Copy, Eq, Encode, Decode, PartialEq, Ord, PartialOrd, Debug, Deserialize, Serialize,
)]
#[serde(transparent)]
pub struct ChainId(#[n(0)] pub u64);

impl AsRef<u64> for ChainId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| {
        f(cell
            .borrow()
            .as_ref()
            .expect("BUG: state is not initialized"))
    })
}

// / Mutates (part of) the current state using `f`.
// /
// / Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|cell| {
        f(cell
            .borrow_mut()
            .as_mut()
            .expect("BUG: state is not initialized"))
    })
}

// State configuration
pub type StableMemory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    pub static STATE: RefCell<Option<State>> = RefCell::new(
        Some(State
             {  minters: BTreeMap::init(minter_memory()), evm_to_icp_txs: BTreeMap::init(evm_to_icp_memory()),
                icp_to_evm_txs: BTreeMap::init(icp_to_evm_memory()), supported_ckerc20_tokens: BTreeMap::init(supported_ckerc20_tokens_memory_id()),
            supported_twin_appic_tokens:BTreeMap::init(supported_appic_tokens_memory_id()) })
    );
}

mod storage_config {
    use super::*;

    thread_local! {
        static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
            MemoryManager::init(DefaultMemoryImpl::default())
        );

    }

    const MINTERS_MEMORY_ID: MemoryId = MemoryId::new(0);

    pub fn minter_memory() -> StableMemory {
        MEMORY_MANAGER.with(|m| m.borrow().get(MINTERS_MEMORY_ID))
    }

    const EVM_TO_ICP_MEMORY_ID: MemoryId = MemoryId::new(1);

    pub fn evm_to_icp_memory() -> StableMemory {
        MEMORY_MANAGER.with(|m| m.borrow().get(EVM_TO_ICP_MEMORY_ID))
    }

    const ICP_TO_EVM_MEMORY_ID: MemoryId = MemoryId::new(2);

    pub fn icp_to_evm_memory() -> StableMemory {
        MEMORY_MANAGER.with(|m| m.borrow().get(ICP_TO_EVM_MEMORY_ID))
    }

    const SUPPORTED_CK_MEMORY_ID: MemoryId = MemoryId::new(3);

    pub fn supported_ckerc20_tokens_memory_id() -> StableMemory {
        MEMORY_MANAGER.with(|m| m.borrow().get(SUPPORTED_CK_MEMORY_ID))
    }

    const SUPPORTED_APPIC_MEMORY_ID: MemoryId = MemoryId::new(4);

    pub fn supported_appic_tokens_memory_id() -> StableMemory {
        MEMORY_MANAGER.with(|m| m.borrow().get(SUPPORTED_APPIC_MEMORY_ID))
    }

    impl Storable for MinterKey {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    impl Storable for Minter {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    impl Storable for EvmToIcpTxIdentifier {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    impl Storable for EvmToIcpStatus {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    impl Storable for EvmToIcpTx {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    impl Storable for IcpToEvmIdentifier {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    impl Storable for IcpToEvmStatus {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    impl Storable for IcpToEvmTx {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    impl Storable for Erc20Identifier {
        fn to_bytes(&self) -> Cow<[u8]> {
            encode(self)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            decode(bytes)
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    fn encode<T: ?Sized + serde::Serialize>(value: &T) -> Cow<[u8]> {
        let mut buf = vec![];
        ciborium::ser::into_writer(value, &mut buf).expect("failed to encode");
        Cow::Owned(buf)
    }

    fn decode<T: serde::de::DeserializeOwned>(bytes: Cow<[u8]>) -> T {
        ciborium::de::from_reader(bytes.as_ref())
            .unwrap_or_else(|e| panic!("failed to decode bytes {}: {e}", hex::encode(bytes)))
    }
}
