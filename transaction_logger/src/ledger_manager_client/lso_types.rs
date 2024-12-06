use crate::{
    scrape_events::NATIVE_ERC20_ADDRESS,
    state::{ChainId, Erc20Identifier},
};
use candid::{CandidType, Deserialize, Nat, Principal};

use super::EvmIcpTwinPairs;

// Dfinity cketh ledger suite orchestrator types

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum OrchestratorArg {
    InitArg(InitArg),
    UpgradeArg(UpgradeArg),
    AddErc20Arg(AddErc20Arg),
}

#[derive(Clone, Eq, PartialEq, Debug, Default, CandidType, Deserialize)]
pub struct InitArg {
    pub more_controller_ids: Vec<Principal>,
    pub minter_id: Option<Principal>,
    pub cycles_management: Option<CyclesManagement>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct UpgradeArg {
    pub git_commit_hash: Option<String>,
    pub ledger_compressed_wasm_hash: Option<String>,
    pub index_compressed_wasm_hash: Option<String>,
    pub archive_compressed_wasm_hash: Option<String>,
    pub cycles_management: Option<UpdateCyclesManagement>,
    pub manage_ledger_suites: Option<Vec<InstalledLedgerSuite>>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct AddErc20Arg {
    pub contract: Erc20Contract,
    pub ledger_init_arg: LedgerInitArg,
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

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct ManagedCanisterIds {
    pub ledger: Option<Principal>,
    pub index: Option<Principal>,
    pub archives: Vec<Principal>,
}

// TODO XC-47: extract type to separate crate since used between ckETH minter and LSO
#[derive(Clone, PartialEq, Debug, CandidType, Deserialize)]
pub struct AddCkErc20Token {
    pub chain_id: Nat,
    pub address: String,
    pub ckerc20_token_symbol: String,
    pub ckerc20_ledger_id: Principal,
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
pub struct InstalledLedgerSuite {
    pub token_symbol: String,
    pub ledger: InstalledCanister,
    pub index: InstalledCanister,
    pub archives: Option<Vec<Principal>>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct InstalledCanister {
    pub canister_id: Principal,
    pub installed_wasm_hash: String,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct ManagedCanisters {
    pub erc20_contract: Erc20Contract,
    pub ckerc20_token_symbol: String,
    pub ledger: Option<ManagedCanisterStatus>,
    pub index: Option<ManagedCanisterStatus>,
    pub archives: Vec<Principal>,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct ManagedLedgerSuite {
    pub token_symbol: String,
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
pub struct OrchestratorInfo {
    pub managed_canisters: Vec<ManagedCanisters>,
    pub cycles_management: CyclesManagement,
    pub more_controller_ids: Vec<Principal>,
    pub minter_id: Option<Principal>,
    pub ledger_suite_version: Option<LedgerSuiteVersion>,
    pub managed_pre_existing_ledger_suites: Option<Vec<ManagedLedgerSuite>>,
}

impl From<OrchestratorInfo> for EvmIcpTwinPairs {
    fn from(value: OrchestratorInfo) -> Self {
        let mut mapped_pairs: Vec<(Erc20Identifier, Principal)> = value
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
        match value.managed_pre_existing_ledger_suites {
            Some(pre_exisiting) => {
                let native_ledger_suite: Vec<(Erc20Identifier, Principal)> = pre_exisiting
                    .into_iter()
                    .filter_map(|canisters| match canisters.ledger {
                        Some(ledger_id) => Some((
                            Erc20Identifier(NATIVE_ERC20_ADDRESS.to_string(), ChainId(1_u64)),
                            ledger_id.into(),
                        )),
                        None => None,
                    })
                    .collect();

                mapped_pairs.extend(native_ledger_suite);
            }
            None => {}
        }
        Self(mapped_pairs)
    }
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
