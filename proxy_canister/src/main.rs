use std::fmt::Debug;

use candid::{CandidType, Principal};
use ic_cdk::update;
use serde::de::DeserializeOwned;
use transaction_logger::{
    endpoints::CandidIcpToken,
    icp_tokens_service::{convert_to_icp_token, MetadataValue},
    minter_client::{CallError, Reason},
};

#[update]
async fn get_icp_token(ledger_id: Principal) -> Result<CandidIcpToken, CallError> {
    match call_canister::<(), Vec<(String, MetadataValue)>>(ledger_id, "icrc1_metadata", ()).await {
        // If error try again.
        Ok(metadata) => {
            if let Some(icp_token) = convert_to_icp_token(ledger_id, metadata, Some(1)).ok() {
                return Ok(CandidIcpToken::from(icp_token));
            } else {
                return Err(CallError {
                    method: "icrc1_metadata".to_string(),
                    reason: Reason::InternalError(
                        "Token Does not have a valid metadata".to_string(),
                    ),
                });
            }
        }
        Err(e) => return Err(e),
    }
}

async fn call_canister<I, O>(canister_id: Principal, method: &str, args: I) -> Result<O, CallError>
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

fn main() {}

ic_cdk::export_candid!();
