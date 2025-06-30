use bincode::Options;

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

const EVM_TOKEN_LIST: MemoryId = MemoryId::new(5);

pub fn evm_token_list_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(EVM_TOKEN_LIST))
}

const ICP_TOKEN_LIST: MemoryId = MemoryId::new(6);

pub fn icp_token_list_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(ICP_TOKEN_LIST))
}

const TWIN_LEDGER_REQUESTS: MemoryId = MemoryId::new(7);

pub fn erc20_twin_ledger_requests_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(TWIN_LEDGER_REQUESTS))
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

impl Storable for EvmToken {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(self)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes)
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for IcpToken {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(self)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes)
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for IcpTokenType {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(self)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes)
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for BridgePair {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(self)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes)
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for Erc20TwinLedgerSuiteRequest {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(self)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes)
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for Erc20TwinLedgerSuiteStatus {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(self)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes)
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for Erc20TwinLedgerSuiteFee {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(self)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes)
    }

    const BOUND: Bound = Bound::Unbounded;
}

fn encode<T: ?Sized + serde::Serialize>(value: &T) -> Cow<[u8]> {
    let bytes = bincode::serialize(value).expect("failed to encode");
    Cow::Owned(bytes)
}

fn decode<T: for<'a> serde::Deserialize<'a>>(bytes: Cow<[u8]>) -> T {
    bincode::deserialize(bytes.as_ref()).unwrap_or_else(|_| panic!("Failed to decode bytes"))
}
