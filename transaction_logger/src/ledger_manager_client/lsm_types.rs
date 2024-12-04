// Appic icrc ledger suite manager types

use candid::{CandidType, Deserialize, Nat, Principal};
use icrc_ledger_types::icrc2::transfer_from::TransferFromError;

use crate::state::Erc20Identifier;

use super::EvmIcpTwinPairs;

type ChainId = Nat;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum LSMarg {
    InitArg(InitArg),
    UpgradeArg(UpgradeArg),
}

#[derive(Clone, Eq, PartialEq, Debug, Default, CandidType, Deserialize)]
pub struct InitArg {
    pub more_controller_ids: Vec<Principal>,
    pub minter_ids: Vec<(ChainId, Principal)>,
    pub cycles_management: Option<CyclesManagement>,
    pub twin_ls_creation_fee_icp_token: Nat,
    pub twin_ls_creation_fee_appic_token: Option<Nat>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct UpdateLedgerSuiteCreationFee {
    pub icp: Nat,
    pub appic: Option<Nat>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct UpgradeArg {
    pub ledger_compressed_wasm_hash: Option<String>,
    pub index_compressed_wasm_hash: Option<String>,
    pub archive_compressed_wasm_hash: Option<String>,
    pub cycles_management: Option<UpdateCyclesManagement>,
    pub twin_ls_creation_fees: Option<UpdateLedgerSuiteCreationFee>,
    pub new_minter_ids: Option<Vec<(ChainId, Principal)>>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct AddErc20Arg {
    pub contract: Erc20Contract,
    pub ledger_init_arg: LedgerInitArg,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub enum AddErc20Error {
    TransferIcpError(TransferFromError),
    InvalidErc20Contract(String),
    ChainIdNotSupported(String),
    Erc20TwinTokenAlreadyExists,
    InternalError(String),
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct Erc20Contract {
    pub chain_id: Nat,
    pub address: String,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize, serde::Serialize)]
pub struct LedgerInitArg {
    pub transfer_fee: Nat,
    pub decimals: u8,
    pub token_name: String,
    pub token_symbol: String,
    pub token_logo: String,
}

#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Debug, CandidType, Deserialize, serde::Serialize,
)]

pub struct CyclesManagement {
    pub cycles_for_ledger_creation: Nat,
    pub cycles_for_archive_creation: Nat,
    pub cycles_for_index_creation: Nat,
    pub cycles_top_up_increment: Nat,
}

#[derive(
    Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, CandidType, Deserialize, serde::Serialize,
)]
pub struct UpdateCyclesManagement {
    pub cycles_for_ledger_creation: Option<Nat>,
    pub cycles_for_archive_creation: Option<Nat>,
    pub cycles_for_index_creation: Option<Nat>,
    pub cycles_top_up_increment: Option<Nat>,
}

#[derive(Clone, PartialEq, Debug, CandidType, Deserialize)]
pub struct InstalledNativeLedgerSuite {
    pub symbol: String,
    pub ledger: Principal,
    pub ledger_wasm_hash: String,
    pub index: Principal,
    pub index_wasm_hash: String,
    pub archives: Vec<Principal>,
    pub chain_id: Nat,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct ManagedCanisterIds {
    pub ledger: Option<Principal>,
    pub index: Option<Principal>,
    pub archives: Vec<Principal>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub enum ManagedCanisterStatus {
    Created {
        canister_id: Principal,
    },
    Installed {
        canister_id: Principal,
        installed_wasm_hash: String,
    },
}

impl From<ManagedCanisterStatus> for Principal {
    fn from(value: ManagedCanisterStatus) -> Self {
        match value {
            ManagedCanisterStatus::Created { canister_id } => canister_id,
            ManagedCanisterStatus::Installed {
                canister_id,
                installed_wasm_hash: _,
            } => canister_id,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct ManagedCanisters {
    pub erc20_contract: Erc20Contract,
    pub twin_erc20_token_symbol: String,
    pub ledger: Option<ManagedCanisterStatus>,
    pub index: Option<ManagedCanisterStatus>,
    pub archives: Vec<Principal>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct LedgerSuiteVersion {
    pub ledger_compressed_wasm_hash: String,
    pub index_compressed_wasm_hash: String,
    pub archive_compressed_wasm_hash: String,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct LedgerManagerInfo {
    pub managed_canisters: Vec<ManagedCanisters>,
    pub cycles_management: CyclesManagement,
    pub more_controller_ids: Vec<Principal>,
    pub minter_ids: Vec<(ChainId, Principal)>,
    pub ledger_suite_version: Option<LedgerSuiteVersion>,
    pub ls_creation_icp_fee: Nat,
    pub ls_creation_appic_fee: Option<Nat>,
}

impl From<LedgerManagerInfo> for EvmIcpTwinPairs {
    fn from(value: LedgerManagerInfo) -> Self {
        let mapped_pairs: Vec<(Erc20Identifier, Principal)> = value
            .managed_canisters
            .into_iter()
            .filter_map(|canisters| match canisters.ledger {
                Some(ledger_id) => Some((
                    Erc20Identifier(
                        canisters.erc20_contract.address,
                        canisters.erc20_contract.chain_id.into(),
                    ),
                    ledger_id.into(),
                )),
                None => None,
            })
            .collect();

        Self(mapped_pairs)
    }
}
