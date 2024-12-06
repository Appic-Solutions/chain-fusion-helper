use crate::state::{EvmToIcpStatus, EvmToIcpTx, IcpToEvmStatus, IcpToEvmTx, Oprator};
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

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]

pub enum Transaction {
    IcpToEvm(CandidIcpToEvm),
    EvmToIcp(CandidEvmToIcp),
}

impl From<CandidIcpToEvm> for Transaction {
    fn from(value: CandidIcpToEvm) -> Self {
        Self::IcpToEvm(value)
    }
}

impl From<CandidEvmToIcp> for Transaction {
    fn from(value: CandidEvmToIcp) -> Self {
        Self::EvmToIcp(value)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CandidIcpToEvm {
    pub transaction_hash: Option<String>,
    pub native_ledger_burn_index: Nat,
    pub withdrawal_amount: Nat,
    pub actual_received: Option<Nat>,
    pub destination: String,
    pub from: Principal,
    pub from_subaccount: Option<[u8; 32]>,
    pub time: u64,
    pub max_transaction_fee: Option<Nat>,
    pub effective_gas_price: Option<Nat>,
    pub gas_used: Option<Nat>,
    pub toatal_gas_spent: Option<Nat>,
    pub erc20_ledger_burn_index: Option<Nat>,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Option<Principal>,
    pub verified: bool,
    pub status: IcpToEvmStatus,
    pub oprator: Oprator,
    pub chain_id: Nat,
}

impl From<IcpToEvmTx> for CandidIcpToEvm {
    fn from(value: IcpToEvmTx) -> Self {
        let IcpToEvmTx {
            transaction_hash,
            native_ledger_burn_index,
            withdrawal_amount,
            actual_received,
            destination,
            from,
            from_subaccount,
            time,
            max_transaction_fee,
            effective_gas_price,
            gas_used,
            toatal_gas_spent,
            erc20_ledger_burn_index,
            erc20_contract_address,
            icrc_ledger_id,
            verified,
            status,
            oprator,
            chain_id,
        } = value;

        Self {
            transaction_hash,
            native_ledger_burn_index,
            withdrawal_amount,
            actual_received,
            destination,
            from,
            from_subaccount,
            time,
            max_transaction_fee,
            effective_gas_price,
            gas_used,
            toatal_gas_spent,
            erc20_ledger_burn_index,
            erc20_contract_address,
            icrc_ledger_id,
            verified,
            status,
            oprator,
            chain_id: Nat::from(chain_id.0),
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CandidEvmToIcp {
    pub from_address: String,
    pub transaction_hash: String,
    pub value: Nat,
    pub block_number: Option<Nat>,
    pub actual_received: Option<Nat>,
    pub principal: Principal,
    pub subaccount: Option<[u8; 32]>,
    pub chain_id: Nat,
    pub total_gas_spent: Option<Nat>,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Option<Principal>,
    pub status: EvmToIcpStatus,
    pub verified: bool,
    pub time: u64,
    pub oprator: Oprator,
}

impl From<EvmToIcpTx> for CandidEvmToIcp {
    fn from(value: EvmToIcpTx) -> Self {
        let EvmToIcpTx {
            from_address,
            transaction_hash,
            value,
            block_number,
            actual_received,
            principal,
            subaccount,
            chain_id,
            total_gas_spent,
            erc20_contract_address,
            icrc_ledger_id,
            status,
            verified,
            time,
            oprator,
        } = value;
        Self {
            from_address,
            transaction_hash,
            value,
            block_number,
            actual_received,
            principal,
            subaccount,
            chain_id: Nat::from(chain_id.0),
            total_gas_spent,
            erc20_contract_address,
            icrc_ledger_id,
            status,
            verified,
            time,
            oprator,
        }
    }
}
