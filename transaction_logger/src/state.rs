use candid::{CandidType, Nat, Principal};
use ic_cdk::trap;
use ic_ethereum_types;
use ic_ethereum_types::Address;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{storable::Bound, Cell, Storable};
use minicbor::{Decode, Encode};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, LowerHex, UpperHex};

use std::fmt::Debug;

use crate::bridge_tx::{
    EvmToIcpSource, EvmToIcpStatus, EvmToIcpTransaction, IcpToEvmSource, IcpToEvmStatus,
    IcpToEvmTransaction,
};
use crate::checked_amount::Erc20Value;

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

// State Definition,
// All types of transactions will be sotred in this stable state
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct State {
    pub evm_to_icp_transactions: BTreeMap<EvmToIcpSource, EvmToIcpTransaction>,
    pub icp_to_evm_transactions: BTreeMap<IcpToEvmSource, IcpToEvmTransaction>,
    pub appic_minters: BTreeMap<ChainId, Principal>,
    pub dfinity_minters: BTreeMap<ChainId, Principal>,
}

impl State {
    pub fn get_appic_minter(&self, chain_id: &ChainId) -> Option<Principal> {
        match self.appic_minters.get(chain_id) {
            Some(minter_id) => Some(minter_id.clone()),
            None => None,
        }
    }

    pub fn get_dfinity_minter(&self, chain_id: &ChainId) -> Option<Principal> {
        match self.dfinity_minters.get(chain_id) {
            Some(minter_id) => Some(minter_id.clone()),
            None => None,
        }
    }

    pub fn record_appic_minter(&mut self, chain_id: ChainId, minter_id: Principal) {
        self.appic_minters.insert(chain_id, minter_id);
    }

    pub fn record_dfinity_minter(&mut self, chain_id: ChainId, minter_id: Principal) {
        self.appic_minters.insert(chain_id, minter_id);
    }

    // Adds a new Evm To Icp Tx
    pub fn record_evm_to_icp_tx(&mut self, tx: EvmToIcpTransaction) {
        self.evm_to_icp_transactions.insert(tx.clone().into(), tx);
    }

    pub fn get_evm_to_icp_tx(&self, source: &EvmToIcpSource) -> Option<EvmToIcpTransaction> {
        match self.evm_to_icp_transactions.get(source) {
            Some(tx) => Some(tx.clone()),
            None => None,
        }
    }

    pub fn update_evm_to_ic_tx_status(
        &mut self,
        tx: &EvmToIcpSource,
        tx_status: EvmToIcpStatus,
        received_icrc_value: Option<Nat>,
    ) {
        if let Some(tx) = self.evm_to_icp_transactions.get_mut(tx) {
            tx.status = tx_status;
            tx.received_icrc_value = received_icrc_value;
        }
    }

    // Adds a new Icp To Evm Tx
    pub fn record_icp_to_evm_tx(&mut self, tx: IcpToEvmTransaction) {
        self.icp_to_evm_transactions.insert(tx.clone().into(), tx);
    }

    pub fn get_icp_to_evm_tx(&self, source: &IcpToEvmSource) -> Option<IcpToEvmTransaction> {
        match self.icp_to_evm_transactions.get(source) {
            Some(tx) => Some(tx.clone()),
            None => None,
        }
    }

    pub fn update_icp_to_evm_tx_status(
        &mut self,
        tx: &IcpToEvmSource,
        tx_status: IcpToEvmStatus,
        received_erc20_value: Option<Erc20Value>,
    ) {
        if let Some(tx) = self.icp_to_evm_transactions.get_mut(tx) {
            tx.status = tx_status;
            tx.received_erc20_value = received_erc20_value;
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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Erc20Token(ChainId, Address);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Serialize, CandidType)]
pub struct IcrcToken(Principal);

impl Erc20Token {
    pub fn new(chain_id: ChainId, address: Address) -> Self {
        Self(chain_id, address)
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.0
    }

    pub fn address(&self) -> &Address {
        &self.1
    }
}

impl From<Nat> for ChainId {
    fn from(value: Nat) -> Self {
        Self(value.0.to_u64().unwrap())
    }
}

// impl TryFrom<Erc20Contract> for Erc20Token {
//     type Error = String;

//     // fn try_from(contract: crate::endpoints::Erc20Contract) -> Result<Self, Self::Error> {
//     //     Ok(Self(
//     //         ChainId(contract.chain_id.0.to_u64().ok_or("chain_id is not u64")?),
//     //         Address::from_str(&contract.address)?,
//     //     ))
//     // }
// }

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ChainId(u64);

impl AsRef<u64> for ChainId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Decode, Deserialize, Encode, Serialize,
)]
#[serde(transparent)]
#[cbor(transparent)]
pub struct Hash(
    #[serde(with = "ic_ethereum_types::serde_data")]
    #[cbor(n(0), with = "minicbor::bytes")]
    pub [u8; 32],
);

impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl LowerHex for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl UpperHex for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode_upper(self.0))
    }
}

impl std::str::FromStr for Hash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("0x") {
            return Err("Ethereum hash doesn't start with 0x".to_string());
        }
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(&s[2..], &mut bytes)
            .map_err(|e| format!("failed to decode hash from hex: {}", e))?;
        Ok(Self(bytes))
    }
}
