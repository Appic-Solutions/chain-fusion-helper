use async_trait::async_trait;
use candid::Principal;

use crate::endpoints::{AddIcpToEvmTx, ValidationError};
use crate::{bridge_tx::Oprator, state::ChainId};
use std::fmt;

use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::CandidType;
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use appic_minter_types::{
    WithdrawalDetail as AppicWithdrawalDetail,
    WithdrawalSearchParameter as AppicWithdrawalSearchParameter,
};
use dfinity_ck_minter_types::{
    WithdrawalDetail as DfinityWithdrawalDetail,
    WithdrawalSearchParameter as DfinityWithdrawalSearchParameter,
};

use num_traits::ToPrimitive;

pub mod appic_minter_types;
pub mod dfinity_ck_minter_types;

#[async_trait]
pub trait Runtime {
    // Making inter canister calls
    async fn call_canister<I, O>(
        &self,
        canister_id: Principal,
        method: &str,
        args: I,
    ) -> Result<O, CallError>
    where
        I: CandidType + Debug + Send + 'static,
        O: CandidType + DeserializeOwned + Debug + 'static;
}

#[derive(Copy, Clone)]
pub struct IcRunTime();

#[async_trait]
impl Runtime for IcRunTime {
    async fn call_canister<I, O>(
        &self,
        canister_id: Principal,
        method: &str,
        args: I,
    ) -> Result<O, CallError>
    where
        I: CandidType + Debug + Send + 'static,
        O: CandidType + DeserializeOwned + Debug + 'static,
    {
        let res: Result<(O,), _> = ic_cdk::api::call::call(canister_id, method, (&args,)).await;

        match res {
            Ok((output,)) => Ok(output),
            Err((code, msg)) => Err(CallError {
                method: method.to_string(),
                reason: Reason::from_reject(code, msg),
            }),
        }
    }
}

pub struct MinterClient {
    runtime: IcRunTime,
    minter_id: Principal,
}

impl MinterClient {
    pub fn new(minter_id: Principal) -> Self {
        Self {
            runtime: IcRunTime(),
            minter_id,
        }
    }

    pub async fn get_appic_witdrawal_status(
        &self,
        tx: &AddIcpToEvmTx,
        minter_id: Principal,
    ) -> Result<Vec<AppicWithdrawalDetail>, ValidationError> {
        self.runtime
            .call_canister::<AppicWithdrawalSearchParameter, Vec<AppicWithdrawalDetail>>(
                minter_id,
                "withdrawal_status",
                AppicWithdrawalSearchParameter::ByWithdrawalId(tx.native_ledger_burn_index),
            )
            .await
            .map_err(|e| ValidationError::InternallError(e.to_string()))
    }

    pub async fn get_dfinity_ck_witdrawal_status(
        &self,
        tx: &AddIcpToEvmTx,
        minter_id: Principal,
    ) -> Result<Vec<DfinityWithdrawalDetail>, ValidationError> {
        self.runtime
            .call_canister::<DfinityWithdrawalSearchParameter, Vec<DfinityWithdrawalDetail>>(
                minter_id,
                "withdrawal_status",
                DfinityWithdrawalSearchParameter::ByWithdrawalId(tx.native_ledger_burn_index),
            )
            .await
            .map_err(|e| ValidationError::InternallError(e.to_string()))
    }
}

/// Represents an error from a management canister call, such as
/// `sign_with_ecdsa`.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CallError {
    pub method: String,
    pub reason: Reason,
}

impl CallError {
    /// Returns the name of the method that resulted in this error.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the failure reason.
    pub fn reason(&self) -> &Reason {
        &self.reason
    }
}

impl fmt::Display for CallError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "management call '{}' failed: {}",
            self.method, self.reason
        )
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
/// The reason for the management call failure.
pub enum Reason {
    /// The canister does not have enough cycles to submit the request.
    OutOfCycles,
    /// The call failed with an error.
    CanisterError(String),
    /// The management canister rejected the signature request (not enough
    /// cycles, the ECDSA subnet is overloaded, etc.).
    Rejected(String),
    /// The call failed with a transient error. Retrying may help.
    TransientInternalError(String),
    /// The call failed with a non-transient error. Retrying will not help.
    InternalError(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfCycles => write!(fmt, "the canister is out of cycles"),
            Self::CanisterError(msg) => write!(fmt, "canister error: {}", msg),
            Self::Rejected(msg) => {
                write!(fmt, "the management canister rejected the call: {}", msg)
            }
            Reason::TransientInternalError(msg) => write!(fmt, "transient internal error: {}", msg),
            Reason::InternalError(msg) => write!(fmt, "internal error: {}", msg),
        }
    }
}

impl Reason {
    pub fn from_reject(reject_code: RejectionCode, reject_message: String) -> Self {
        match reject_code {
            RejectionCode::SysTransient => Self::TransientInternalError(reject_message),
            RejectionCode::CanisterError => Self::CanisterError(reject_message),
            RejectionCode::CanisterReject => Self::Rejected(reject_message),
            RejectionCode::NoError
            | RejectionCode::SysFatal
            | RejectionCode::DestinationInvalid
            | RejectionCode::Unknown => Self::InternalError(format!(
                "rejection code: {:?}, rejection message: {}",
                reject_code, reject_message
            )),
        }
    }
}