// Appic minter types

use candid::{CandidType, Deserialize, Nat, Principal};
use icrc_ledger_types::icrc1::account::Account;

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct WithdrawalDetail {
    pub withdrawal_id: u64,
    pub recipient_address: String,
    pub from: Principal,
    pub from_subaccount: Option<[u8; 32]>,
    pub token_symbol: String,
    pub withdrawal_amount: Nat,
    pub max_transaction_fee: Option<Nat>,
    pub status: WithdrawalStatus,
}

#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum WithdrawalSearchParameter {
    ByWithdrawalId(u64),
    ByRecipient(String),
    BySenderAccount(Account),
}

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Transaction {
    pub transaction_hash: String,
}

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum WithdrawalStatus {
    Pending,
    TxCreated,
    TxSent(Transaction),
    TxFinalized(TxFinalizedStatus),
}

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum TxFinalizedStatus {
    Success {
        transaction_hash: String,
        effective_transaction_fee: Option<Nat>,
    },
    PendingReimbursement(Transaction),
    Reimbursed {
        transaction_hash: String,
        reimbursed_amount: Nat,
        reimbursed_in_block: Nat,
    },
}
