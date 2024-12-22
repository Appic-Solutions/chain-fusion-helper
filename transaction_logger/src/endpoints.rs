use std::str::FromStr;

use crate::state::{
    nat_to_u128, read_state, ChainId, Erc20Identifier, Erc20TwinLedgerSuiteFee,
    Erc20TwinLedgerSuiteRequest, Erc20TwinLedgerSuiteStatus, EvmToIcpStatus, EvmToIcpTx, EvmToken,
    IcpToEvmStatus, IcpToEvmTx, IcpToken, IcpTokenType, Operator,
};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_ethereum_types::Address;
use serde::Serialize;

#[derive(Debug, CandidType, Deserialize)]
pub struct Icrc28TrustedOriginsResponse {
    pub trusted_origins: Vec<String>,
}

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
    pub operator: Operator,
}

pub type CandidChainId = Nat;
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum AddEvmToIcpTxError {
    TxAlreadyExsits,
    InvalidTokenPairs,
    ChinNotSupported,
    InvalidTokenContract,
    InvalidAddress,
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
    pub operator: Operator,
    pub chain_id: CandidChainId,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum AddIcpToEvmTxError {
    TxAlreadyExsits,
    InvalidTokenPairs,
    ChinNotSupported,
    InvalidDestination,
    InvalidTokenContract,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct MinterArgs {
    pub chain_id: CandidChainId,
    pub minter_id: Principal,
    pub operator: Operator,
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
    pub operator: Operator,
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

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct GetTxParams {
    pub chain_id: CandidChainId,
    pub search_param: TransactionSearchParam,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum TransactionSearchParam {
    TxHash(String),
    TxWithdrawalId(Nat),
    TxMintId(Nat),
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
    pub total_gas_spent: Option<Nat>,
    pub erc20_ledger_burn_index: Option<Nat>,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Option<Principal>,
    pub verified: bool,
    pub status: IcpToEvmStatus,
    pub operator: Operator,
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
            total_gas_spent,
            erc20_ledger_burn_index,
            erc20_contract_address,
            icrc_ledger_id,
            verified,
            status,
            operator,
            chain_id,
        } = value;

        Self {
            transaction_hash,
            native_ledger_burn_index: native_ledger_burn_index.get().into(),
            withdrawal_amount: withdrawal_amount.into(),
            actual_received: actual_received.map(|actual_received| actual_received.into()),
            destination: destination.to_string(),
            from,
            from_subaccount,
            time,
            max_transaction_fee: max_transaction_fee
                .map(|max_transaction_fee| max_transaction_fee.into()),
            effective_gas_price: effective_gas_price
                .map(|effective_gas_price| effective_gas_price.into()),
            gas_used: gas_used.map(|gas_used| gas_used.into()),
            total_gas_spent: total_gas_spent.map(|total_gas_spent| total_gas_spent.into()),
            erc20_ledger_burn_index: erc20_ledger_burn_index.map(|index| index.get().into()),
            erc20_contract_address: erc20_contract_address.to_string(),
            icrc_ledger_id,
            verified,
            status,
            operator,
            chain_id: Nat::from(chain_id),
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CandidEvmToIcp {
    pub from_address: String,
    pub transaction_hash: String,
    pub value: Nat,
    pub block_number: Option<Nat>,
    pub ledger_mint_index: Option<Nat>,
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
    pub operator: Operator,
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
            operator,
            ledger_mint_index,
        } = value;
        Self {
            from_address: from_address.to_string(),
            transaction_hash,
            value: value.into(),
            block_number: block_number.map(|blokc_number| blokc_number.into()),
            ledger_mint_index: ledger_mint_index
                .map(|ledger_mint_index| ledger_mint_index.get().into()),
            actual_received: actual_received.map(|actual_received| actual_received.into()),
            principal,
            subaccount,
            chain_id: Nat::from(chain_id),
            total_gas_spent: total_gas_spent.map(|total_gas_spent| total_gas_spent.into()),
            erc20_contract_address: erc20_contract_address.to_string(),
            icrc_ledger_id,
            status,
            verified,
            time,
            operator,
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct GetEvmTokenArgs {
    pub address: String,
    pub chain_id: CandidChainId,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CandidEvmToken {
    pub chain_id: CandidChainId,
    pub erc20_contract_address: String,
    pub name: String,
    pub decimals: u8,
    pub symbol: String,
    pub logo: String,
}

impl From<EvmToken> for CandidEvmToken {
    fn from(value: EvmToken) -> Self {
        Self {
            chain_id: value.chain_id.into(),
            erc20_contract_address: value.erc20_contract_address.to_string(),
            name: value.name,
            decimals: value.decimals,
            symbol: value.symbol,
            logo: value.logo,
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct GetIcpTokenArgs {
    pub ledger_id: Principal,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CandidIcpToken {
    pub ledger_id: Principal,
    pub name: String,
    pub decimals: u8,
    pub symbol: String,
    pub token_type: IcpTokenType,
    pub logo: String,
    pub usd_price: String,
    pub fee: Nat,
    pub rank: Option<u32>,
}

impl From<IcpToken> for CandidIcpToken {
    fn from(value: IcpToken) -> Self {
        Self {
            ledger_id: value.ledger_id,
            name: value.name,
            decimals: value.decimals,
            symbol: value.symbol,
            logo: value.logo,
            usd_price: value.usd_price,
            token_type: value.token_type,
            fee: value.fee.into(),
            rank: value.rank,
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TokenPair {
    pub evm_token: CandidEvmToken,
    pub icp_token: CandidIcpToken,
    pub operator: Operator,
}

#[derive(Clone, CandidType, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub enum CandidErc20TwinLedgerSuiteStatus {
    PendingApproval,
    Created,
    Installed,
}

impl From<CandidErc20TwinLedgerSuiteStatus> for Erc20TwinLedgerSuiteStatus {
    fn from(value: CandidErc20TwinLedgerSuiteStatus) -> Self {
        match value {
            CandidErc20TwinLedgerSuiteStatus::PendingApproval => Self::PendingApproval,
            CandidErc20TwinLedgerSuiteStatus::Created => Self::Created,
            CandidErc20TwinLedgerSuiteStatus::Installed => Self::Installed,
        }
    }
}

impl From<Erc20TwinLedgerSuiteStatus> for CandidErc20TwinLedgerSuiteStatus {
    fn from(value: Erc20TwinLedgerSuiteStatus) -> Self {
        match value {
            Erc20TwinLedgerSuiteStatus::PendingApproval => Self::PendingApproval,
            Erc20TwinLedgerSuiteStatus::Created => Self::Created,
            Erc20TwinLedgerSuiteStatus::Installed => Self::Installed,
        }
    }
}

#[derive(Clone, CandidType, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub enum CandidErc20TwinLedgerSuiteFee {
    Icp(Nat),
    Appic(Nat),
}

impl From<CandidErc20TwinLedgerSuiteFee> for Erc20TwinLedgerSuiteFee {
    fn from(value: CandidErc20TwinLedgerSuiteFee) -> Self {
        match value {
            CandidErc20TwinLedgerSuiteFee::Icp(nat) => Self::Icp(nat_to_u128(&nat)),
            CandidErc20TwinLedgerSuiteFee::Appic(nat) => Self::Appic(nat_to_u128(&nat)),
        }
    }
}
impl From<Erc20TwinLedgerSuiteFee> for CandidErc20TwinLedgerSuiteFee {
    fn from(value: Erc20TwinLedgerSuiteFee) -> Self {
        match value {
            Erc20TwinLedgerSuiteFee::Icp(amount) => Self::Icp(amount.into()),
            Erc20TwinLedgerSuiteFee::Appic(amount) => Self::Appic(amount.into()),
        }
    }
}
#[derive(CandidType, Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub struct CandidAddErc20TwinLedgerSuiteRequest {
    creator: Principal,
    evm_token_contract: String,
    evm_token_chain_id: CandidChainId,
    icp_token_name: String,
    icp_token_symbol: String,
    icp_ledger_id: Option<Principal>,
    status: CandidErc20TwinLedgerSuiteStatus,
    created_at: u64,
    fee_charged: CandidErc20TwinLedgerSuiteFee,
}

impl From<&CandidAddErc20TwinLedgerSuiteRequest> for Erc20Identifier {
    fn from(value: &CandidAddErc20TwinLedgerSuiteRequest) -> Self {
        let erc20_address = Address::from_str(&value.evm_token_contract).unwrap();
        let chain_id = ChainId::from(&value.evm_token_chain_id);
        Erc20Identifier::new(&erc20_address, chain_id)
    }
}

impl From<CandidAddErc20TwinLedgerSuiteRequest> for Erc20TwinLedgerSuiteRequest {
    fn from(value: CandidAddErc20TwinLedgerSuiteRequest) -> Self {
        let erc20_address = Address::from_str(&value.evm_token_contract).unwrap();
        let chain_id = ChainId::from(&value.evm_token_chain_id);
        let identifier = Erc20Identifier::new(&erc20_address, chain_id);
        let evm_token = read_state(|s| s.get_evm_token_by_identifier(&identifier));
        let icp_token = read_state(|s| {
            s.get_icp_token_by_principal(&value.icp_ledger_id.unwrap_or(Principal::anonymous()))
        });

        Self {
            creator: value.creator,
            evm_token,
            ledger_id: value.icp_ledger_id,
            icp_token_name: value.icp_token_name,
            icp_token_symbol: value.icp_token_symbol,
            icp_token,
            status: value.status.into(),
            created_at: value.created_at,
            fee_charged: value.fee_charged.into(),
            erc20_contract_address: erc20_address,
            chain_id,
        }
    }
}

#[derive(CandidType, Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub struct CandidLedgerSuiteRequest {
    pub creator: Principal,
    pub evm_token: Option<CandidEvmToken>,
    pub icp_token: Option<CandidIcpToken>,
    pub erc20_contract: String,
    pub chain_id: CandidChainId,
    pub status: CandidErc20TwinLedgerSuiteStatus,
    pub created_at: u64,
    pub fee_charged: CandidErc20TwinLedgerSuiteFee,
}
