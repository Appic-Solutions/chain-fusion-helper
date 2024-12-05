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
    pub chain_id: u64,
    pub total_gas_spent: Nat,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Principal,
    pub time: u64,
    pub oprator: Oprator,
}

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
    pub time: u64,
    pub max_transaction_fee: Nat,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Principal,
    pub oprator: Oprator,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum AddIcpToEvmTxError {
    TxAlreadyExsits,
    InvalidTokenPairs,
    ChinNotSupported,
}

pub enum LoggerArgs {
    Init(),
    Upgrade(),
}
