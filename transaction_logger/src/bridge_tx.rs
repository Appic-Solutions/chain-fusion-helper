use std::{ops::Add, str::FromStr};

use crate::{
    checked_amount::Erc20Value,
    state::{ChainId, Hash, IcrcToken},
};
use candid::{CandidType, Nat, Principal};
use ic_ethereum_types::Address;
use icrc_ledger_types::icrc1::account::Account;
use serde::{Deserialize, Serialize};

use crate::minter_clinet::appic_minter_types::{
    TxFinalizedStatus as AppicTxFinalizedStatus, WithdrawalDetail as AppicWithdrawalDetail,
    WithdrawalStatus as AppicWithdrawalStatus,
};
use crate::minter_clinet::dfinity_ck_minter_types::{
    TxFinalizedStatus as DfinityTxFinalizedStatus, WithdrawalDetail as DfinityWithdrawalDetail,
    WithdrawalStatus as DfinityWithdrawalStatus,
};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum EvmToIcpStatus {
    TxMined,
    TxScrapedByMinter,
    TxFailed,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize, CandidType)]
pub enum IcpToEvmStatus {
    TxPending,
    TxCreated,
    TxSigned,
    TxSent,
    TxFinalized,
    TxPendingReimburse,
    TxReimbursed,
    TxFailed,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize, CandidType)]
pub enum Oprator {
    Dfinity,
    Appic,
}

type NativeBurnIndex = u64;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct EvmToIcpTransaction {
    pub transaction_hash: Hash,
    pub chain_id: ChainId,
    pub from_token: Address,
    pub to_token: IcrcToken,
    pub time: u64,
    pub erc20_value: Erc20Value,
    pub received_icrc_value: Option<Nat>,
    pub estimated_icrc_value: Nat,
    pub status: EvmToIcpStatus,
    pub from: Address,
    pub destintion: Account,
    pub oprator: Oprator,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Eq, PartialOrd, Ord)]
pub struct EvmToIcpSource(Hash, ChainId);

impl From<EvmToIcpTransaction> for EvmToIcpSource {
    fn from(value: EvmToIcpTransaction) -> Self {
        Self(value.transaction_hash, value.chain_id)
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Eq, PartialOrd, Ord)]
pub struct IcpToEvmSource(pub NativeBurnIndex, pub ChainId);

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct IcpToEvmTransaction {
    pub chain_id: Option<ChainId>,
    pub transaction_hash: Option<Hash>,
    pub native_burn_index: NativeBurnIndex,
    pub from_token: Option<IcrcToken>,
    pub to_token: Option<Address>,
    pub time: Option<u64>,
    pub icrc_value: Nat,
    pub received_erc20_value: Option<Erc20Value>,
    pub estimated_erc20_value: Erc20Value,
    pub status: IcpToEvmStatus,
    pub from: Account,
    pub destintion: Address,
    pub oprator: Oprator,
}

impl From<IcpToEvmTransaction> for IcpToEvmSource {
    fn from(value: IcpToEvmTransaction) -> Self {
        Self(
            value.native_burn_index,
            value.chain_id.expect("Chain Id Should not be empty"),
        )
    }
}

// Dfinity Withdrawal detail conversion
impl From<DfinityWithdrawalStatus> for IcpToEvmStatus {
    fn from(value: DfinityWithdrawalStatus) -> Self {
        match value {
            DfinityWithdrawalStatus::Pending => Self::TxPending,
            DfinityWithdrawalStatus::TxCreated => Self::TxCreated,
            DfinityWithdrawalStatus::TxSent(_eth_transaction) => Self::TxSent,
            DfinityWithdrawalStatus::TxFinalized(tx_finalized_status) => {
                match tx_finalized_status {
                    DfinityTxFinalizedStatus::Success {
                        transaction_hash: _,
                        effective_transaction_fee: _,
                    } => Self::TxFinalized,
                    DfinityTxFinalizedStatus::PendingReimbursement(_eth_transaction) => {
                        Self::TxPendingReimburse
                    }
                    DfinityTxFinalizedStatus::Reimbursed {
                        transaction_hash: _,
                        reimbursed_amount: _,
                        reimbursed_in_block: _,
                    } => Self::TxPendingReimburse,
                }
            }
        }
    }
}

impl From<DfinityWithdrawalDetail> for IcpToEvmTransaction {
    fn from(value: DfinityWithdrawalDetail) -> Self {
        let status = IcpToEvmStatus::from(value.status.clone());

        Self {
            transaction_hash: hash_from_dfinty_tx(value.clone()),
            native_burn_index: value.withdrawal_id.clone(),
            from_token: None,
            to_token: None,
            time: None,
            icrc_value: value.withdrawal_amount.clone(),
            received_erc20_value: None,
            estimated_erc20_value: estimate_erc20_value(
                value.withdrawal_amount.clone(),
                value.max_transaction_fee.clone(),
            ),
            status,
            from: Account {
                owner: value.from.clone(),
                subaccount: value.from_subaccount.clone(),
            },
            destintion: Address::from_str(&value.recipient_address)
                .expect("Should not fail parsing minter returned string to address"),
            oprator: Oprator::Dfinity,
            chain_id: None,
        }
    }
}

fn hash_from_dfinty_tx(tx: DfinityWithdrawalDetail) -> Option<Hash> {
    match tx.status {
        DfinityWithdrawalStatus::TxFinalized(tx_finalized_status) => match tx_finalized_status {
            DfinityTxFinalizedStatus::Success {
                transaction_hash,
                effective_transaction_fee: _,
            } => Some(
                Hash::from_str(&transaction_hash)
                    .expect("Conversion from minter to hash should not fail"),
            ),
            _ => None,
        },
        _ => None,
    }
}

// Appic witdrawaldetail conversion
impl From<AppicWithdrawalStatus> for IcpToEvmStatus {
    fn from(value: AppicWithdrawalStatus) -> Self {
        match value {
            AppicWithdrawalStatus::Pending => Self::TxPending,
            AppicWithdrawalStatus::TxCreated => Self::TxCreated,
            AppicWithdrawalStatus::TxSent(_eth_transaction) => Self::TxSent,
            AppicWithdrawalStatus::TxFinalized(tx_finalized_status) => match tx_finalized_status {
                AppicTxFinalizedStatus::Success {
                    transaction_hash: _,
                    effective_transaction_fee: _,
                } => Self::TxFinalized,
                AppicTxFinalizedStatus::PendingReimbursement(_eth_transaction) => {
                    Self::TxPendingReimburse
                }
                AppicTxFinalizedStatus::Reimbursed {
                    transaction_hash: _,
                    reimbursed_amount: _,
                    reimbursed_in_block: _,
                } => Self::TxPendingReimburse,
            },
        }
    }
}

impl From<AppicWithdrawalDetail> for IcpToEvmTransaction {
    fn from(value: AppicWithdrawalDetail) -> Self {
        let status = IcpToEvmStatus::from(value.status.clone());

        Self {
            transaction_hash: hash_from_appic_tx(value.clone()),
            native_burn_index: value.withdrawal_id.clone(),
            from_token: None,
            to_token: None,
            time: None,
            icrc_value: value.withdrawal_amount.clone(),
            received_erc20_value: None,
            estimated_erc20_value: estimate_erc20_value(
                value.withdrawal_amount.clone(),
                value.max_transaction_fee.clone(),
            ),
            status,
            from: Account {
                owner: value.from.clone(),
                subaccount: value.from_subaccount.clone(),
            },
            destintion: Address::from_str(&value.recipient_address)
                .expect("Should not fail parsing minter returned string to address"),
            oprator: Oprator::Appic,
            chain_id: None,
        }
    }
}

fn hash_from_appic_tx(tx: AppicWithdrawalDetail) -> Option<Hash> {
    match tx.status {
        AppicWithdrawalStatus::TxFinalized(tx_finalized_status) => match tx_finalized_status {
            AppicTxFinalizedStatus::Success {
                transaction_hash,
                effective_transaction_fee: _,
            } => Some(
                Hash::from_str(&transaction_hash)
                    .expect("Conversion from minter to hash should not fail"),
            ),
            _ => None,
        },
        _ => None,
    }
}

fn estimate_erc20_value(withdrawal_amount: Nat, max_transaction_fee: Option<Nat>) -> Erc20Value {
    Erc20Value::try_from(withdrawal_amount)
        .expect("Should not overflow")
        .checked_sub(
            Erc20Value::try_from(max_transaction_fee.unwrap_or(Nat::from(0_u64)))
                .expect("Should not fail converting either gas fee or 0 to erc20 value"),
        )
        .expect("Erc20 value should never go below 0")
}
