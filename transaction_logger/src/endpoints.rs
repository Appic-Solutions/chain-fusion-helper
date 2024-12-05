use crate::state::Oprator;
use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;
// Transactions for Evm to Icp
// unique identifier = transaction hash and chain id
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct AddEvmToIcpTx {
    pub from_address: String,
    pub transaction_hash: String,
    pub value: Nat,
    pub principal: Principal,
    pub subaccount: Option<[u8; 32]>,
    pub chain_id: CandidChainId,
    pub total_gas_spent: Nat,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Principal,
    pub time: Nat,
    pub oprator: Oprator,
}

pub type CandidChainId = Nat;
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum AddEvmToIcpTxError {
    TxAlreadyExsits,
    InvalidTokenPairs,
    ChinNotSupported,
}

// Transactions for icp to evm
// unique identifier= native ledger bunr index and chain id
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct AddIcpToEvmTx {
    pub native_ledger_burn_index: Nat,
    pub withdrawal_amount: Nat,
    pub destination: String,
    pub from: Principal,
    pub from_subaccount: Option<[u8; 32]>,
    pub time: Nat,
    pub max_transaction_fee: Nat,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Principal,
    pub oprator: Oprator,
    pub chain_id: CandidChainId,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum AddIcpToEvmTxError {
    TxAlreadyExsits,
    InvalidTokenPairs,
    ChinNotSupported,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct MinterArgs {
    pub chain_id: CandidChainId,
    pub minter_id: Principal,
    pub oprator: Oprator,
    pub last_observed_event: Nat,
    pub last_scraped_event: Nat,
    pub last_scraped_event_time: Nat,
    pub evm_to_icp_fee: Nat,
    pub icp_to_evm_fee: Nat,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UpdateMinterArgs {
    pub chain_id: CandidChainId,
    pub minter_id: Principal,
    pub evm_to_icp_fee: Nat,
    pub icp_to_evm_fee: Nat,
    pub oprator: Oprator,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct InitArgs {
    pub minters: Vec<MinterArgs>,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UpgradeArg {
    pub new_minters: Option<Vec<MinterArgs>>,
    pub update_minters: Option<Vec<UpdateMinterArgs>>,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum LoggerArgs {
    Init(InitArgs),
    Upgrade(UpgradeArg),
}
