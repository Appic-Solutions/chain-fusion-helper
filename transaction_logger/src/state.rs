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

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Deserialize, Serialize)]
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

impl From<&Minter> for MinterKey {
    fn from(value: &Minter) -> Self {
        Self(value.chain_id.clone(), value.oprator.clone())
    }
}
// State Definition,
// All types of transactions will be sotred in this stable state
#[derive(Clone, PartialEq, Debug, PartialOrd, Eq, Ord, Deserialize, Serialize)]
pub struct State {
    pub minters: BTreeMap<MinterKey, Minter>,
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

    // pub fn get_dfinity_minter(&self, chain_id: &ChainId) -> Option<Principal> {
    //     match self.dfinity_minters.get(chain_id) {
    //         Some(minter_id) => Some(minter_id.clone()),
    //         None => None,
    //     }
    // }

    // pub fn record_appic_minter(&mut self, chain_id: ChainId, minter_id: Principal) {
    //     self.appic_minters.insert(chain_id, minter_id);
    // }

    // pub fn record_dfinity_minter(&mut self, chain_id: ChainId, minter_id: Principal) {
    //     self.appic_minters.insert(chain_id, minter_id);
    // }
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
