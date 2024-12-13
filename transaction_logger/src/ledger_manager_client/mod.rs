use candid::Principal;
use lsm_types::LedgerManagerInfo;
use lso_types::OrchestratorInfo;

use crate::{
    minter_clinet::{IcRunTime, Runtime},
    state::{Erc20Identifier, Operator},
};

// Appic
pub mod lsm_types;

// Dfinity
pub mod lso_types;

pub struct LsClient {
    pub runtime: IcRunTime,
    pub id: Principal,
    pub operator: Operator,
}

pub struct EvmIcpTwinPairs(Vec<(Erc20Identifier, Principal)>);

impl EvmIcpTwinPairs {
    pub fn get_token_pairs_iter(self) -> std::vec::IntoIter<(Erc20Identifier, Principal)> {
        self.0.into_iter()
    }
}

impl LsClient {
    pub fn new(id: Principal, operator: Operator) -> Self {
        Self {
            runtime: IcRunTime(),
            id,
            operator,
        }
    }

    pub async fn get_erc20_list(&self) -> Result<EvmIcpTwinPairs, crate::minter_clinet::CallError> {
        match self.operator {
            Operator::DfinityCkEthMinter => {
                let result = self
                    .runtime
                    .call_canister::<_, OrchestratorInfo>(self.id, "get_orchestrator_info", ())
                    .await?;

                Ok(EvmIcpTwinPairs::from(result))
            }
            Operator::AppicMinter => {
                let result = self
                    .runtime
                    .call_canister::<_, LedgerManagerInfo>(self.id, "get_lsm_info", ())
                    .await?;

                Ok(EvmIcpTwinPairs::from(result))
            }
        }
    }
}
