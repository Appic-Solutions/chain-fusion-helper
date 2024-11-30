use candid::{Nat, Principal};
use ic_cdk::trap;
use ic_ethereum_types;
use ic_ethereum_types::Address;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{storable::Bound, Cell, Storable};
use minicbor::{Decode, Encode};
use num_traits::ToPrimitive;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter, LowerHex, UpperHex};

use std::fmt::Debug;
use std::ops::Sub;

use icrc_ledger_types::icrc1::account::Account;

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

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum TransactionStrategy {
    EVMtoICP,
    ICPtoEVM,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum EvmToIcpStatus {
    TransactionMined,
    TransactionScrapedByMinter,
    TransactionFailed,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum IcpToEvmStatus {
    TransactionCreated,
    TransactionSigned,
    TransactionSent,
    TransactionFinalized,
    TransactionReimbursed,
    TransactionFailed,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum Oprator {
    Dfinity,
    Appic,
}

type Subaccount = [u8; 32];

type NativeBurnIndex = Nat;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct EvmToIcpTransaction {
    pub transaction_hash: Hash,
    pub from_token: Erc20Token,
    pub to_token: IcrcToken,
    pub time: u64,
    pub erc20_value: Erc20Value,
    pub received_icrc_value: Nat,
    pub deposit_status: EvmToIcpStatus,
    pub from: Address,
    pub destintion: Account,
    pub oprator: Oprator,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Eq, PartialOrd, Ord)]
pub struct EvmToIcpSource(Hash, ChainId);

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Eq, PartialOrd, Ord)]
pub struct IcpToEvmSource(NativeBurnIndex, Account, ChainId);

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct IcpToEvmTransaction {
    pub transaction_hash: Option<Hash>,
    pub from_token: IcrcToken,
    pub to_token: Erc20Token,
    pub time: u64,
    pub icrc_value: Erc20Value,
    pub received_erc20_value: Nat,
    pub deposit_status: IcpToEvmStatus,
    pub from: Account,
    pub destintion: Principal,
    pub oprator: Oprator,
}

// State Definition,
// All types of transactions will be sotred in this stable state
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct State {
    pub evm_to_icp_transactions: BTreeMap<EvmToIcpSource, EvmToIcpTransaction>,
    pub icp_to_evm_transactions: BTreeMap<IcpToEvmSource, IcpToEvmTransaction>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Erc20Token(ChainId, Address);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
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
