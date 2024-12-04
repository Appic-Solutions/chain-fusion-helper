use candid::{CandidType, Nat, Principal};
use ic_cdk::trap;
use ic_ethereum_types;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{storable::Bound, Cell, Storable};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use std::cell::RefCell;
use std::collections::BTreeMap;

use std::fmt::Debug;

use crate::minter_clinet::appic_minter_types::events::{
    EventSource, TransactionReceipt, TransactionStatus,
};

#[derive(Clone, CandidType, PartialEq, PartialOrd, Eq, Ord, Debug, Deserialize, Serialize)]
pub enum Oprator {
    DfinityCkEthMinter,
    AppicMinter,
}

#[derive(Clone, PartialEq, Ord, PartialOrd, Eq, Debug, Deserialize, Serialize)]
pub struct Minter {
    pub id: Principal,
    pub last_observed_event: u64,
    pub last_scraped_event: u64,
    pub oprator: Oprator,
    pub chain_id: ChainId,
}

impl Minter {
    pub fn update_last_observed_event(&mut self, event: u64) {
        self.last_observed_event = event
    }

    pub fn update_last_scraped_event(&mut self, event: u64) {
        self.last_scraped_event = event
    }
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct MinterKey(ChainId, Oprator);

impl MinterKey {
    pub fn oprator(&self) -> Oprator {
        self.1.clone()
    }

    pub fn chain_id(&self) -> ChainId {
        self.0.clone()
    }
}

impl From<&Minter> for MinterKey {
    fn from(value: &Minter) -> Self {
        Self(value.chain_id.clone(), value.oprator.clone())
    }
}

type TransactionHash = String;

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct EvmToIcpTxIdentifier(TransactionHash, ChainId);

impl EvmToIcpTxIdentifier {
    pub fn new(transaction_hash: &TransactionHash, chain_id: &ChainId) -> Self {
        EvmToIcpTxIdentifier(transaction_hash.clone(), chain_id.clone())
    }
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum EvmToIcpStatus {
    PendingVerification,
    Accepted,
    Minted,
    Invalid(String),
    Quarantined,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct EvmToIcpTx {
    pub from_address: String,
    pub transaction_hash: TransactionHash,
    pub value: Nat,
    pub block_number: Option<Nat>,
    pub actual_received: Option<Nat>,
    pub principal: Principal,
    pub subaccount: Option<[u8; 32]>,
    pub chain_id: ChainId,
    pub total_gas_spent: Option<Nat>,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Option<Principal>,
    pub status: EvmToIcpStatus,
    pub verified: bool,
    pub time: u64,
    pub oprator: Oprator,
}

pub type NativeLedgerBurnIndex = Nat;

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct IcpToEvmIdentifier(NativeLedgerBurnIndex, ChainId);
impl IcpToEvmIdentifier {
    pub fn new(native_ledger_burn_index: &NativeLedgerBurnIndex, chain_id: &ChainId) -> Self {
        IcpToEvmIdentifier(native_ledger_burn_index.clone(), chain_id.clone())
    }
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum IcpToEvmStatus {
    PendingVerification,
    Accepted,
    Created,
    SignedTransaction,
    FinalizedTransaction,
    ReplacedTransaction,
    Reimbursed,
    QuarantinedReimbursement,
    Successful,
    Failed,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct IcpToEvmTx {
    pub transaction_hash: Option<TransactionHash>,
    pub native_ledger_burn_index: NativeLedgerBurnIndex,
    pub withdrawal_amount: Nat,
    pub destination: String,
    pub from: Principal,
    pub from_subaccount: Option<[u8; 32]>,
    pub time: u64,
    pub max_transaction_fee: Option<Nat>,
    pub effective_gas_price: Option<Nat>,
    pub gas_used: Option<Nat>,
    pub toatal_gas_spent: Option<Nat>,
    pub erc20_ledger_burn_index: Option<Nat>,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Option<Principal>,
    pub verified: bool,
    pub status: IcpToEvmStatus,
    pub oprator: Oprator,
}

type Erc20Contract = String;
// State Definition,
// All types of transactions will be sotred in this stable state
#[derive(Clone, PartialEq, Debug, PartialOrd, Eq, Ord, Deserialize, Serialize)]
pub struct State {
    pub minters: BTreeMap<MinterKey, Minter>,
    pub evm_to_icp_txs: BTreeMap<EvmToIcpTxIdentifier, EvmToIcpTx>,
    pub icp_to_evm_txs: BTreeMap<IcpToEvmIdentifier, IcpToEvmTx>,
    pub supported_ckerc20_tokens: BTreeMap<Erc20Contract, Principal>,
    pub supported_twin_appic_tokens: BTreeMap<Erc20Contract, Principal>,
}

impl State {
    pub fn get_minter(&self, minter_key: &MinterKey) -> Option<Minter> {
        match self.minters.get(minter_key) {
            Some(minter) => Some(minter.clone()),
            None => None,
        }
    }

    pub fn get_minter_mut(&mut self, minter_key: &MinterKey) -> Option<&mut Minter> {
        self.minters.get_mut(minter_key)
    }

    pub fn get_minters_iter(&self) -> std::collections::btree_map::IntoIter<MinterKey, Minter> {
        self.minters.clone().into_iter()
    }

    pub fn get_minters_mut_iter(
        &mut self,
    ) -> std::collections::btree_map::IterMut<'_, MinterKey, Minter> {
        self.minters.iter_mut()
    }

    pub fn record_minter(&mut self, minter: Minter) {
        self.minters.insert(MinterKey::from(&minter), minter);
    }

    pub fn get_icrc_twin_for_erc20(
        &self,
        erc20_contract_address: &Erc20Contract,
        oprator: &Oprator,
    ) -> Option<Principal> {
        match oprator {
            Oprator::AppicMinter => self
                .supported_twin_appic_tokens
                .get(erc20_contract_address)
                .map(|token_principal| token_principal.clone()),
            Oprator::DfinityCkEthMinter => self
                .supported_ckerc20_tokens
                .get(erc20_contract_address)
                .map(|token_principal| token_principal.clone()),
        }
    }

    pub fn record_evm_to_icp_tx(&mut self) {}

    pub fn if_evm_to_icp_tx_exists(&self, identifier: &EvmToIcpTxIdentifier) -> bool {
        self.evm_to_icp_txs.get(identifier).is_some()
    }

    pub fn record_new_evm_to_icp(&mut self, identifier: EvmToIcpTxIdentifier, tx: EvmToIcpTx) {
        self.evm_to_icp_txs.insert(identifier, tx);
    }

    pub fn record_accepted_evm_to_icp(
        &mut self,
        identifier: &EvmToIcpTxIdentifier,
        transaction_hash: TransactionHash,
        block_number: Nat,
        from_address: String,
        value: Nat,
        principal: Principal,
        erc20_contract_address: String,
        subaccount: Option<[u8; 32]>,
        chain_id: &ChainId,
        oprator: &Oprator,
    ) {
        if let Some(tx) = self.evm_to_icp_txs.get_mut(identifier) {
            *tx = EvmToIcpTx {
                verified: true,
                block_number: Some(block_number),
                from_address,
                value,
                principal,
                erc20_contract_address,
                subaccount,
                status: EvmToIcpStatus::Accepted,
                ..tx.clone() // Copies the remaining fields
            };
        } else {
            match oprator {
                Oprator::DfinityCkEthMinter => {}
                Oprator::AppicMinter => {
                    self.record_new_evm_to_icp(
                        identifier.clone(),
                        EvmToIcpTx {
                            from_address,
                            transaction_hash: transaction_hash,
                            value,
                            block_number: Some(block_number),
                            actual_received: None,
                            principal,
                            subaccount,
                            chain_id: chain_id.clone(),
                            total_gas_spent: None,
                            erc20_contract_address: erc20_contract_address.clone(),
                            icrc_ledger_id: self
                                .get_icrc_twin_for_erc20(&erc20_contract_address, oprator),
                            status: EvmToIcpStatus::Accepted,
                            verified: true,
                            time: ic_cdk::api::time(),
                            oprator: oprator.clone(),
                        },
                    );
                }
            }
        }
    }

    pub fn record_minted_evm_to_icp(
        &mut self,
        identifier: &EvmToIcpTxIdentifier,
        erc20_contract_address: String,
    ) {
        if let Some(tx) = self.evm_to_icp_txs.get_mut(identifier) {
            *tx = EvmToIcpTx {
                erc20_contract_address,
                status: EvmToIcpStatus::Minted,
                ..tx.clone() // Copies the remaining fields
            };
        }
    }

    pub fn record_invalid_evm_to_icp(&mut self, identifier: &EvmToIcpTxIdentifier, reason: String) {
        if let Some(tx) = self.evm_to_icp_txs.get_mut(identifier) {
            *tx = EvmToIcpTx {
                status: EvmToIcpStatus::Invalid(reason),
                ..tx.clone() // Copies the remaining fields
            };
        }
    }

    pub fn record_quarantined_evm_to_icp(&mut self, identifier: &EvmToIcpTxIdentifier) {
        if let Some(tx) = self.evm_to_icp_txs.get_mut(identifier) {
            *tx = EvmToIcpTx {
                status: EvmToIcpStatus::Quarantined,
                ..tx.clone() // Copies the remaining fields
            };
        }
    }

    pub fn record_new_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier, tx: IcpToEvmTx) {
        self.icp_to_evm_txs.insert(identifier, tx);
    }

    pub fn record_accepted_icp_to_evm(
        &mut self,
        identifier: &IcpToEvmIdentifier,
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
    ) {
        if let Some(tx) = self.icp_to_evm_txs.get_mut(identifier) {
            *tx = IcpToEvmTx {
                verified: true,
                max_transaction_fee: max_transaction_fee,
                withdrawal_amount,
                erc20_contract_address,
                destination,
                native_ledger_burn_index,
                erc20_ledger_burn_index,
                from,
                from_subaccount,
                status: IcpToEvmStatus::Accepted,
                ..tx.clone()
            }
        } else {
            match oprator {
                Oprator::AppicMinter => {
                    let new_tx = IcpToEvmTx {
                        native_ledger_burn_index,
                        withdrawal_amount,
                        destination,
                        from,
                        from_subaccount,
                        time: created_at.unwrap_or(ic_cdk::api::time()),
                        max_transaction_fee: max_transaction_fee,
                        erc20_ledger_burn_index,
                        icrc_ledger_id: self
                            .get_icrc_twin_for_erc20(&erc20_contract_address, &oprator),
                        erc20_contract_address,
                        verified: true,
                        status: IcpToEvmStatus::Accepted,
                        oprator,
                        effective_gas_price: None,
                        gas_used: None,
                        toatal_gas_spent: None,
                        transaction_hash: None,
                    };

                    self.record_new_icp_to_evm(identifier.clone(), new_tx);
                }
                Oprator::DfinityCkEthMinter => {}
            }
        }
    }

    pub fn record_created_icp_to_evm(&mut self, identifier: &IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get_mut(identifier) {
            *tx = IcpToEvmTx {
                status: IcpToEvmStatus::Created,
                ..tx.clone()
            }
        }
    }

    pub fn record_signed_icp_to_evm(&mut self, identifier: &IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get_mut(identifier) {
            *tx = IcpToEvmTx {
                status: IcpToEvmStatus::SignedTransaction,
                ..tx.clone()
            }
        }
    }

    pub fn record_replaced_icp_to_evm(&mut self, identifier: &IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get_mut(identifier) {
            *tx = IcpToEvmTx {
                status: IcpToEvmStatus::ReplacedTransaction,
                ..tx.clone()
            }
        }
    }

    pub fn record_finalized_icp_to_evm(
        &mut self,
        identifier: &IcpToEvmIdentifier,
        receipt: TransactionReceipt,
    ) {
        if let Some(tx) = self.icp_to_evm_txs.get_mut(identifier) {
            let status = match receipt.status {
                TransactionStatus::Success => IcpToEvmStatus::Successful,
                TransactionStatus::Failure => IcpToEvmStatus::Failed,
            };
            *tx = IcpToEvmTx {
                status,
                transaction_hash: Some(receipt.transaction_hash),
                gas_used: Some(receipt.gas_used.clone()),
                effective_gas_price: Some(receipt.effective_gas_price.clone()),
                toatal_gas_spent: Some(receipt.gas_used * receipt.effective_gas_price),
                ..tx.clone()
            }
        }
    }

    pub fn record_reimbursed_icp_to_evm(&mut self, identifier: &IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get_mut(identifier) {
            *tx = IcpToEvmTx {
                status: IcpToEvmStatus::Reimbursed,
                ..tx.clone()
            }
        }
    }

    pub fn record_quarantined_reimbursed_icp_to_evm(&mut self, identifier: &IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get_mut(identifier) {
            *tx = IcpToEvmTx {
                status: IcpToEvmStatus::QuarantinedReimbursement,
                ..tx.clone()
            }
        }
    }
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| f(cell.borrow().get().expect_initialized()))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        let mut state = borrowed.get().expect_initialized().clone();
        let result = f(&mut state);
        borrowed
            .set(ConfigState::Initialized(state))
            .expect("failed to write state in stable cell");
        result
    })
}

pub fn init_state(state: State) {
    STATE.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        assert_eq!(
            borrowed.get(),
            &ConfigState::Uninitialized,
            "BUG: State is already initialized and has value {:?}",
            borrowed.get()
        );
        borrowed
            .set(ConfigState::Initialized(state))
            .expect("failed to initialize state in stable cell")
    });
}

impl From<Nat> for ChainId {
    fn from(value: Nat) -> Self {
        Self(value.0.to_u64().unwrap())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ChainId(u64);

impl AsRef<u64> for ChainId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

// State configuration
pub type StableMemory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);

pub fn state_memory() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(STATE_MEMORY_ID))
}

thread_local! {
    pub static STATE: RefCell<Cell<ConfigState, StableMemory>> = RefCell::new(Cell::init(
   state_memory(), ConfigState::default())
    .expect("failed to initialize stable cell for state"));
}

/// Configuration state of the lsm.
#[derive(Clone, PartialEq, Debug, Default)]
enum ConfigState {
    #[default]
    Uninitialized,
    // This state is only used between wasm module initialization and init().
    Initialized(State),
}

impl ConfigState {
    fn expect_initialized(&self) -> &State {
        match &self {
            ConfigState::Uninitialized => trap("BUG: state not initialized"),
            ConfigState::Initialized(s) => s,
        }
    }
}

impl Storable for ConfigState {
    fn to_bytes(&self) -> Cow<[u8]> {
        match &self {
            ConfigState::Uninitialized => Cow::Borrowed(&[]),
            ConfigState::Initialized(config) => Cow::Owned(encode(config)),
        }
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        if bytes.is_empty() {
            return ConfigState::Uninitialized;
        }
        ConfigState::Initialized(decode(bytes.as_ref()))
    }

    const BOUND: Bound = Bound::Unbounded;
}

fn encode<S: ?Sized + serde::Serialize>(state: &S) -> Vec<u8> {
    let mut buf = vec![];
    ciborium::ser::into_writer(state, &mut buf).expect("failed to encode state");
    buf
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    ciborium::de::from_reader(bytes)
        .unwrap_or_else(|e| panic!("failed to decode state bytes {}: {e}", hex::encode(bytes)))
}
