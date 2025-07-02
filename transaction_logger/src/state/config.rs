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
