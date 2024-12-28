// Token service updates list of available tokens on icp on a daily basis
// Tokens are fetched from icpswap token list canister and sonic swap

use candid::Principal;
use ic_canister_log::log;
use icp_swap_token_type::TokensListResult;
use icp_swap_usd_node_types::PublicTokenOverview;
use sonic_swap_types::TokenInfoWithType;

use crate::{
    logs::INFO,
    minter_client::{CallError, IcRunTime, Reason, Runtime},
    numeric::Erc20TokenAmount,
    state::{IcpToken, IcpTokenType},
};

mod icp_swap_token_type;
mod icp_swap_usd_node_types;
mod sonic_swap_types;

const SONIC_ID: &str = "3xwpq-ziaaa-aaaah-qcn4a-cai";
const ICP_SWAP_ID: &str = "k37c6-riaaa-aaaag-qcyza-cai";
const ICP_SWAP_NODE: &str = "ggzvv-5qaaa-aaaag-qck7a-cai";
pub struct TokenService {
    runtime: IcRunTime,
}

impl TokenService {
    pub fn new() -> Self {
        Self {
            runtime: IcRunTime {},
        }
    }

    pub async fn get_sonic_tokens(&self) -> Vec<IcpToken> {
        // Get all sonic supported tokens
        let sonic_tokens = self
            .runtime
            .call_canister::<(), Vec<TokenInfoWithType>>(
                Principal::from_text(SONIC_ID).unwrap(),
                "getSupportedTokenList",
                (),
            )
            .await;

        // Error handling
        match sonic_tokens {
            Ok(tokens) => {
                // Map and filter tokens into Valid IcpToken
                tokens
                    .into_iter()
                    .map(|token| IcpToken::from(token))
                    .filter(|mapped_token| mapped_token.ledger_id != Principal::anonymous())
                    .collect()
            }
            Err(e) => {
                log!(INFO, "Failed To get icp_swap tokens for {}", e);

                return vec![];
            }
        }
    }

    pub async fn get_icp_swap_tokens(&self) -> Vec<IcpToken> {
        // Get all icp swap supported tokens

        let icp_swap_tokens = self
            .runtime
            .call_canister::<(), TokensListResult>(
                Principal::from_text(ICP_SWAP_ID).unwrap(),
                "getList",
                (),
            )
            .await;

        // Error handling
        match icp_swap_tokens {
            Ok(tokens) => match tokens {
                // Map and filter tokens into Valid IcpToken
                TokensListResult::Ok(tokens) => tokens
                    .into_iter()
                    .map(|token| IcpToken::from(token))
                    .filter(|mapped_token| {
                        mapped_token.ledger_id != Principal::anonymous()
                            && mapped_token.decimals != 0
                            && mapped_token.fee != Erc20TokenAmount::ZERO
                    })
                    .collect(),
                TokensListResult::Err(e) => {
                    log!(INFO, "Failed To get icp_swap tokens for {}", e);

                    return vec![];
                }
            },
            Err(e) => {
                log!(INFO, "Failed To get icp_swap tokens for {}", e);

                return vec![];
            }
        }
    }

    // Validate tokens on icp
    // There might be some tokens that are not valid for some reasons
    // 1. Tokens wasm was removed and canister is not functional
    // 2. Token canister ran out of cycles
    // 3. Tokens was a scam And the canister is removed
    // This function checks if the tokens canister is still working by simply calling the icrc1_decimals() and if canister
    // Returns a value, it means the token canister is still operating

    pub async fn validate_token(
        &self,
        ledger_id: Principal,
        token_type: &IcpTokenType,
    ) -> Result<u8, CallError> {
        match token_type {
            IcpTokenType::ICRC1 | IcpTokenType::ICRC2 | IcpTokenType::ICRC3 => {
                let result = self
                    .runtime
                    .call_canister::<(), u8>(ledger_id, "icrc1_decimals", ())
                    .await;
                // If error try again.
                match result {
                    Ok(decimals) => Ok(decimals),
                    Err(_e) => {
                        self.runtime
                            .call_canister::<(), u8>(ledger_id, "icrc1_decimals", ())
                            .await
                    }
                }
            }
            IcpTokenType::DIP20 => {
                let result = self
                    .runtime
                    .call_canister::<(), u8>(ledger_id, "decimals", ())
                    .await;

                // If error try again.
                match result {
                    Ok(decimals) => Ok(decimals),
                    Err(_e) => {
                        self.runtime
                            .call_canister::<(), u8>(ledger_id, "decimals", ())
                            .await
                    }
                }
            }
            IcpTokenType::Other(_) => Err(CallError {
                method: "Token Type Not supported".to_string(),
                reason: Reason::InternalError("Token Type Not supported".to_string()),
            }),
        }
    }

    pub async fn get_icp_swap_tokens_with_usd_price(
        &self,
    ) -> Result<Vec<PublicTokenOverview>, CallError> {
        self.runtime
            .call_canister::<(), Vec<PublicTokenOverview>>(
                Principal::from_text(ICP_SWAP_NODE).unwrap(),
                "getAllTokens",
                (),
            )
            .await
    }
}
