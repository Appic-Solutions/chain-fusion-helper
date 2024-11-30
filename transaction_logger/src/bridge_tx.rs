use crate::{
    checked_amount::Erc20Value,
    state::{ChainId, Erc20Token, Hash, IcrcToken},
};
use candid::{Nat, Principal};
use ic_ethereum_types::Address;
use icrc_ledger_types::icrc1::account::Account;
use serde::{Deserialize, Serialize};

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

type NativeBurnIndex = Nat;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct EvmToIcpTransaction {
    pub transaction_hash: Hash,
    pub chain_id: ChainId,
    pub from_token: Erc20Token,
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
pub struct IcpToEvmSource(NativeBurnIndex, ChainId);

impl From<IcpToEvmTransaction> for IcpToEvmSource {
    fn from(value: IcpToEvmTransaction) -> Self {
        Self(value.native_burn_index, value.chain_id)
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct IcpToEvmTransaction {
    pub transaction_hash: Option<Hash>,
    pub native_burn_index: NativeBurnIndex,
    pub chain_id: ChainId,
    pub from_token: IcrcToken,
    pub to_token: Erc20Token,
    pub time: u64,
    pub icrc_value: Nat,
    pub received_erc20_value: Option<Erc20Value>,
    pub estimated_erc20_value: Erc20Value,
    pub status: IcpToEvmStatus,
    pub from: Account,
    pub destintion: Principal,
    pub oprator: Oprator,
}
