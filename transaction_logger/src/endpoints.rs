use std::str::FromStr;

use async_trait::async_trait;
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_ethereum_types::Address;
use icrc_ledger_types::icrc1::account::Account;

use crate::{
    bridge_tx::{IcpToEvmSource, IcpToEvmStatus, IcpToEvmTransaction, Oprator},
    minter_clinet::MinterClient,
    state::{read_state, ChainId, Erc20Token, IcrcToken},
};

use crate::minter_clinet::appic_minter_types::{
    TxFinalizedStatus as AppicTxFinalizedStatus, WithdrawalDetail as AppicWithdrawalDetail,
    WithdrawalSearchParameter as AppicWithdrawalSearchParameter,
    WithdrawalStatus as AppicWithdrawalStatus,
};
use crate::minter_clinet::dfinity_ck_minter_types::{
    TxFinalizedStatus as DfinityTxFinalizedStatus, WithdrawalDetail as DfinityWithdrawalDetail,
    WithdrawalSearchParameter as DfinityWithdrawalSearchParameter,
    WithdrawalStatus as DfinityWithdrawalStatus,
};

type CandidChainId = Nat;

type Erc20Address = String;

#[async_trait]
pub trait Validation<O> {
    async fn validate_tx(self) -> Result<O, ValidationError>;
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub enum ValidationError {
    MinterNotfound,
    InvalidDestination,
    InvalidCaller,
    TxNotFound,
    TxDataMismath,
    TxAlreadyExsits,
    TokenNotFound,
    InternallError(String),
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct AddIcpToEvmTx {
    pub native_ledger_burn_index: u64,
    pub erc20_ledger_burn_index: Option<Nat>,
    pub chain_id: Nat,
    pub from_token: IcrcToken,
    pub to_token: Erc20Address,
    pub oprator: Oprator,
}

// Validation to make sure transaction is correct
#[async_trait]
impl Validation<IcpToEvmTransaction> for AddIcpToEvmTx {
    // Validation steps:
    // 1: Validate tx data
    // 2: Validate if tx already exist.
    // 3: Validate if the chain is supported either by dfinity or appic
    // 4: Check if target transaction exists in the minters state
    async fn validate_tx(self) -> Result<IcpToEvmTransaction, ValidationError> {
        // 1st Validate to token address
        let to_token =
            Address::from_str(&self.to_token).map_err(|_e| ValidationError::TokenNotFound)?;

        let chain_id = ChainId::from(self.chain_id.clone());

        // 2nd Validate if tx already exist
        let validation_result: () = read_state(|s| {
            let source = IcpToEvmSource(self.native_ledger_burn_index.clone(), chain_id.clone());
            if s.get_icp_to_evm_tx(&source).is_some() {
                Err(ValidationError::TxAlreadyExsits)
            } else {
                Ok(())
            }
        })?;

        // 3rd validation step
        let minter_id = match self.oprator {
            Oprator::Dfinity => read_state(|s| s.get_dfinity_minter(&chain_id))
                .ok_or_else(|| ValidationError::MinterNotfound),
            Oprator::Appic => read_state(|s| s.get_appic_minter(&chain_id))
                .ok_or_else(|| ValidationError::MinterNotfound),
        }?;

        // 4th validation step
        let minter_client = MinterClient::new(minter_id);

        match self.oprator {
            Oprator::Dfinity => {
                let dfinity_result: Vec<DfinityWithdrawalDetail> = minter_client
                    .get_dfinity_ck_witdrawal_status(&self, minter_id)
                    .await?;
                match dfinity_result.len() {
                    0 => return Err(ValidationError::TxNotFound),
                    1 => {
                        let ck_tx: DfinityWithdrawalDetail = dfinity_result[0].clone();
                        let mut icp_to_evm_tx = IcpToEvmTransaction::from(ck_tx);
                        icp_to_evm_tx.from_token = Some(self.from_token);
                        icp_to_evm_tx.to_token = Some(to_token);
                        icp_to_evm_tx.time = Some(ic_cdk::api::time());
                        icp_to_evm_tx.chain_id = Some(chain_id);
                        return Ok(icp_to_evm_tx);
                    }
                    _ => return Err(ValidationError::TxDataMismath),
                };
            }
            Oprator::Appic => {
                let appic_result: Vec<AppicWithdrawalDetail> = minter_client
                    .get_appic_witdrawal_status(&self, minter_id)
                    .await?;
                match appic_result.len() {
                    0 => return Err(ValidationError::TxNotFound),
                    1 => {
                        let ck_tx: AppicWithdrawalDetail = appic_result[0].clone();
                        let mut icp_to_evm_tx = IcpToEvmTransaction::from(ck_tx);
                        icp_to_evm_tx.from_token = Some(self.from_token);
                        icp_to_evm_tx.to_token = Some(to_token);
                        icp_to_evm_tx.time = Some(ic_cdk::api::time());
                        icp_to_evm_tx.chain_id = Some(chain_id);
                        return Ok(icp_to_evm_tx);
                    }
                    _ => return Err(ValidationError::TxDataMismath),
                };
            }
        }

        // Since searched for tx by ledger_burn index there should only be one tx returned from minter
    }
}
