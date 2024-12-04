pub mod appic_minter_types;
pub mod dfinity_ck_minter_types;
pub mod event_conversion;
use async_trait::async_trait;
use candid::Principal;

use std::fmt;

use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::CandidType;
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use num_traits::ToPrimitive;

use crate::state::{ChainId, Oprator};

use crate::state::Minter;

use appic_minter_types::{
    events::GetEventsArg as AppicGetEventsArg, events::GetEventsResult as AppicGetEventsResult,
};
use dfinity_ck_minter_types::{
    events::GetEventsArg as DfinityCkGetEventsArg,
    events::GetEventsResult as DfinityCkGetEventsResult,
};
use event_conversion::{Events, Reduce};

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
    oprator: Oprator,
}

impl From<&Minter> for MinterClient {
    fn from(value: &Minter) -> Self {
        Self {
            runtime: IcRunTime(),
            minter_id: value.id,
            oprator: value.oprator.clone(),
        }
    }
}

impl MinterClient {
    pub fn new(minter_id: &Principal, oprator: &Oprator) -> Self {
        Self {
            runtime: IcRunTime(),
            minter_id: *minter_id,
            oprator: oprator.clone(),
        }
    }

    // Get total evetns count
    pub async fn get_total_events_count(&self) -> u64 {
        // Get total events count
        let toatl_events_count = match self.oprator {
            Oprator::DfinityCkEthMinter => {
                self.runtime
                    .call_canister::<DfinityCkGetEventsArg, DfinityCkGetEventsResult>(
                        self.minter_id,
                        "get_events",
                        DfinityCkGetEventsArg {
                            start: 0,
                            length: 0,
                        },
                    )
                    .await
                    .expect("Call should not fail. will retry in next interval")
                    .total_event_count
            }
            Oprator::AppicMinter => {
                self.runtime
                    .call_canister::<AppicGetEventsArg, AppicGetEventsResult>(
                        self.minter_id,
                        "get_events",
                        AppicGetEventsArg {
                            start: 0,
                            length: 0,
                        },
                    )
                    .await
                    .expect("Call should not fail. will retry in next interval")
                    .total_event_count
            }
        };

        toatl_events_count
    }

    // scrape events
    pub async fn scrape_events(&self, from_event: u64, length: u64) -> Result<Events, CallError> {
        match self.oprator {
            Oprator::DfinityCkEthMinter => self
                .runtime
                .call_canister::<DfinityCkGetEventsArg, DfinityCkGetEventsResult>(
                    self.minter_id,
                    "get_events",
                    DfinityCkGetEventsArg {
                        start: from_event,
                        length,
                    },
                )
                .await
                .map(|response| response.reduce()),
            Oprator::AppicMinter => self
                .runtime
                .call_canister::<AppicGetEventsArg, AppicGetEventsResult>(
                    self.minter_id,
                    "get_events",
                    AppicGetEventsArg {
                        start: from_event,
                        length,
                    },
                )
                .await
                .map(|response| response.reduce()),
        }
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
