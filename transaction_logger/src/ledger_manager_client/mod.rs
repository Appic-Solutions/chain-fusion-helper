use candid::Principal;
use lsm_types::Erc20Contract;
use lso_types::OrchestratorInfo;

use crate::{
    minter_clinet::{IcRunTime, Runtime},
    state::{Erc20Identifier, Oprator},
};

// Appic
pub mod lsm_types;

// Dfinity
pub mod lso_types;

pub struct LsClient {
    pub runtime: IcRunTime,
    pub id: Principal,
    pub oprator: Oprator,
}

pub struct EvmIcpTwinPairs(Vec<(Erc20Identifier, Principal)>);

impl LsClient {
    pub async fn get_erc20_list(&self) -> Result<(), crate::minter_clinet::CallError> {
        match self.oprator {
            Oprator::DfinityCkEthMinter => {
                let result = self
                    .runtime
                    .call_canister::<_, OrchestratorInfo>(self.id, "get_orchestrator_info", ())
                    .await?;

                let mut token_pairs: Vec<(Erc20Identifier, Principal)> = vec![];
            }
            Oprator::AppicMinter => todo!(),
        };
        Ok(())
    }
}
