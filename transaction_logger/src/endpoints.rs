use std::str::FromStr;

use crate::address::Address;
use crate::state::dex::types::{DexAction, PoolId, PositionKey, SwapType};
use crate::state::nat_to_u64;
use crate::state::{
    checked_nat_to_erc20_amount, nat_to_u128,
    types::{
        ChainId, Erc20TwinLedgerSuiteFee, Erc20TwinLedgerSuiteStatus, EvmToIcpStatus, EvmToIcpTx,
        EvmToken, IcpToEvmStatus, IcpToEvmTx, IcpToken, IcpTokenType, Operator,
    },
};
use candid::{CandidType, Deserialize, Int, Nat, Principal};
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
    pub operator: Operator,
}

pub type CandidChainId = Nat;
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum AddEvmToIcpTxError {
    TxAlreadyExists,
    InvalidTokenPairs,
    ChainNotSupported,
    InvalidTokenContract,
    InvalidAddress,
}

// Transactions for icp to evm
// unique identifier= native ledger burn index and chain id
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct AddIcpToEvmTx {
    pub native_ledger_burn_index: Nat,
    pub withdrawal_amount: Nat,
    pub destination: String,
    pub from: Principal,
    pub from_subaccount: Option<[u8; 32]>,
    pub max_transaction_fee: Nat,
    pub erc20_contract_address: String,
    pub icrc_ledger_id: Principal,
    pub operator: Operator,
    pub chain_id: CandidChainId,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum AddIcpToEvmTxError {
    TxAlreadyExists,
    InvalidTokenPairs,
    ChainNotSupported,
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
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UpdateMinterArgs {
    pub chain_id: CandidChainId,
    pub minter_id: Principal,
    pub operator: Operator,
    pub last_observed_event: Option<Nat>,
    pub last_scraped_event: Option<Nat>,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct InitArgs {
    pub minters: Vec<MinterArgs>,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UpgradeArg {
    pub new_minters: Option<Vec<MinterArgs>>,
    pub update_minters: Option<Vec<UpdateMinterArgs>>,
    pub update_latest_observed_dex_event: Option<Nat>,
    pub update_latest_scraped_dex_event: Option<Nat>,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum LoggerArgs {
    Init(InitArgs),
    Upgrade(UpgradeArg),
}

#[derive(CandidType, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Transaction {
    IcpToEvm(CandidIcpToEvm),
    EvmToIcp(CandidEvmToIcp),
    DexAction(CandidDexAction),
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

#[derive(
    CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash,
)]
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
            native_ledger_burn_index: native_ledger_burn_index.into(),
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
            erc20_ledger_burn_index: erc20_ledger_burn_index.map(|index| index.into()),
            erc20_contract_address: erc20_contract_address.to_string(),
            icrc_ledger_id,
            verified,
            status,
            operator,
            chain_id: Nat::from(chain_id),
        }
    }
}

#[derive(
    CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash,
)]
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
            block_number: block_number.map(|block_number| block_number.into()),
            ledger_mint_index: ledger_mint_index.map(|ledger_mint_index| ledger_mint_index.into()),
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
    pub is_wrapped_icrc: bool,
    pub usd_price: Option<String>,
    pub cmc_id: Option<Nat>,
    pub volume_usd_24h: Option<String>,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TopVolumeTokens {
    pub chain: ChainId,
    pub tokens: Vec<CandidEvmToken>,
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct EvmSearchQuery {
    // symbol, name or contract address
    pub query: String,
    pub chain_id: ChainId,
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
            is_wrapped_icrc: value.is_wrapped_icrc,
            usd_price: value.usd_price,
            cmc_id: value.cmc_id.map(Nat::from),
            volume_usd_24h: value.volume_usd_24h,
        }
    }
}

impl From<CandidEvmToken> for EvmToken {
    fn from(value: CandidEvmToken) -> Self {
        Self {
            chain_id: ChainId::from(&value.chain_id),
            erc20_contract_address: Address::from_str(&value.erc20_contract_address)
                .expect("Invalid contract address"),
            name: value.name,
            decimals: value.decimals,
            symbol: value.symbol,
            logo: value.logo,
            is_wrapped_icrc: value.is_wrapped_icrc,
            usd_price: value.usd_price,
            cmc_id: value.cmc_id.map(|id| nat_to_u64(&id)),
            volume_usd_24h: value.volume_usd_24h,
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
    pub listed_on_appic_dex: Option<bool>,
}

impl From<IcpToken> for CandidIcpToken {
    fn from(value: IcpToken) -> Self {
        let logo = if value.logo.starts_with("https://") {
            value.logo
        } else {
            format!(
                "https://zjydy-zyaaa-aaaaj-qnfka-cai.raw.icp0.io/logo/{}",
                value.ledger_id
            )
        };
        Self {
            ledger_id: value.ledger_id,
            name: value.name,
            decimals: value.decimals,
            symbol: value.symbol,
            logo,
            usd_price: value.usd_price,
            token_type: value.token_type,
            fee: value.fee.into(),
            rank: value.rank,
            listed_on_appic_dex: value.listed_on_appic_dex,
        }
    }
}

impl From<CandidIcpToken> for IcpToken {
    fn from(value: CandidIcpToken) -> Self {
        Self {
            ledger_id: value.ledger_id,
            name: value.name,
            decimals: value.decimals,
            symbol: value.symbol,
            logo: value.logo,
            usd_price: value.usd_price,
            token_type: value.token_type,
            fee: checked_nat_to_erc20_amount(value.fee).unwrap(),
            rank: value.rank,
            listed_on_appic_dex: value.listed_on_appic_dex,
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

#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct CandidPoolId {
    pub token0: Principal, // Token0 identifier
    pub token1: Principal, // Token1 identifier
    pub fee: Nat,          // Fee tier (e.g., 500 for 0.05%)
}

#[derive(CandidType, Hash, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CandidPositionKey {
    pub owner: Principal,
    pub pool_id: CandidPoolId,
    pub tick_lower: Int,
    pub tick_upper: Int,
}

#[derive(CandidType, Hash, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum CandidSwapType {
    ExactOutput(Vec<CandidPoolId>),
    ExactInput(Vec<CandidPoolId>),
    ExactOutputSingle(CandidPoolId),
    ExactInputSingle(CandidPoolId),
}

/// The event describing the  minter state transition.
#[derive(CandidType, Hash, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum CandidDexAction {
    CreatedPool {
        token0: Principal,
        token1: Principal,
        pool_fee: u32,
        timestamp: u64,
    },
    MintedPosition {
        created_position: CandidPositionKey,
        liquidity: Nat,
        amount0_paid: Nat,
        amount1_paid: Nat,
        timestamp: u64,
    },
    IncreasedLiquidity {
        modified_position: CandidPositionKey,
        liquidity_delta: Nat,
        amount0_paid: Nat,
        amount1_paid: Nat,
        timestamp: u64,
    },
    BurntPosition {
        burnt_position: CandidPositionKey,
        liquidity: Nat,
        amount0_received: Nat,
        amount1_received: Nat,
        timestamp: u64,
    },
    DecreasedLiquidity {
        modified_position: CandidPositionKey,
        liquidity_delta: Nat,
        amount0_received: Nat,
        amount1_received: Nat,
        timestamp: u64,
    },
    CollectedFees {
        position: CandidPositionKey,
        amount0_collected: Nat,
        amount1_collected: Nat,
        timestamp: u64,
    },
    Swap {
        final_amount_in: Nat,
        final_amount_out: Nat,
        swap_type: CandidSwapType,
        timestamp: u64,
        token_in: Principal,
        token_out: Principal,
    },
}

impl From<PositionKey> for CandidPositionKey {
    fn from(value: PositionKey) -> Self {
        Self {
            owner: value.owner,
            pool_id: value.pool_id.into(),
            tick_lower: value.tick_lower.into(),
            tick_upper: value.tick_upper.into(),
        }
    }
}
impl From<PoolId> for CandidPoolId {
    fn from(value: PoolId) -> Self {
        Self {
            token0: value.token0,
            token1: value.token1,
            fee: value.fee.into(),
        }
    }
}

impl From<SwapType> for CandidSwapType {
    fn from(value: SwapType) -> Self {
        match value {
            SwapType::ExactOutput(candid_pool_ids) => CandidSwapType::ExactOutput(
                candid_pool_ids
                    .into_iter()
                    .map(|pool_id| pool_id.into())
                    .collect(),
            ),
            SwapType::ExactInput(candid_pool_ids) => CandidSwapType::ExactInput(
                candid_pool_ids
                    .into_iter()
                    .map(|pool_id| pool_id.into())
                    .collect(),
            ),
            SwapType::ExactOutputSingle(candid_pool_id) => {
                CandidSwapType::ExactOutputSingle(candid_pool_id.into())
            }
            SwapType::ExactInputSingle(candid_pool_id) => {
                CandidSwapType::ExactInputSingle(candid_pool_id.into())
            }
        }
    }
}

impl From<DexAction> for CandidDexAction {
    fn from(value: DexAction) -> Self {
        match value {
            DexAction::CreatedPool {
                token0,
                token1,
                pool_fee,
                timestamp,
            } => CandidDexAction::CreatedPool {
                token0,
                token1,
                pool_fee,
                timestamp,
            },
            DexAction::MintedPosition {
                created_position,
                liquidity,
                amount0_paid,
                amount1_paid,
                timestamp,
            } => CandidDexAction::MintedPosition {
                created_position: created_position.into(),
                liquidity: liquidity.into(),
                amount0_paid: amount0_paid.into(),
                amount1_paid: amount1_paid.into(),
                timestamp,
            },
            DexAction::IncreasedLiquidity {
                modified_position,
                liquidity_delta,
                amount0_paid,
                amount1_paid,
                timestamp,
            } => CandidDexAction::IncreasedLiquidity {
                modified_position: modified_position.into(),
                liquidity_delta: liquidity_delta.into(),
                amount0_paid: amount0_paid.into(),
                amount1_paid: amount1_paid.into(),
                timestamp,
            },
            DexAction::BurntPosition {
                burnt_position,
                liquidity,
                amount0_received,
                amount1_received,
                timestamp,
            } => CandidDexAction::BurntPosition {
                burnt_position: burnt_position.into(),
                liquidity: liquidity.into(),
                amount0_received: amount0_received.into(),
                amount1_received: amount1_received.into(),
                timestamp,
            },
            DexAction::DecreasedLiquidity {
                modified_position,
                liquidity_delta,
                amount0_received,
                amount1_received,
                timestamp,
            } => CandidDexAction::DecreasedLiquidity {
                modified_position: modified_position.into(),
                liquidity_delta: liquidity_delta.into(),
                amount0_received: amount0_received.into(),
                amount1_received: amount1_received.into(),
                timestamp,
            },
            DexAction::CollectedFees {
                position,
                amount0_collected,
                amount1_collected,
                timestamp,
            } => CandidDexAction::CollectedFees {
                position: position.into(),
                amount0_collected: amount0_collected.into(),
                amount1_collected: amount1_collected.into(),
                timestamp,
            },
            DexAction::Swap {
                final_amount_in,
                final_amount_out,
                swap_type,
                timestamp,
                token_in,
                token_out,
            } => CandidDexAction::Swap {
                final_amount_in: final_amount_in.into(),
                final_amount_out: final_amount_out.into(),
                swap_type: swap_type.into(),
                timestamp,
                token_in,
                token_out,
            },
        }
    }
}
