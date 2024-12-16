// Token service updates list of available tokens on icp on a daily basis
// Tokens are fetched from icpswap token list canister and sonic swap

use candid::Principal;
use ic_canister_log::log;
use icp_swap_token_type::TokensListResult;
use sonic_swap_types::TokenInfoWithType;

use crate::{
    logs::INFO,
    minter_clinet::{CallError, IcRunTime, Runtime},
    state::{IcpToken, IcpTokenType},
};

mod icp_swap_token_type;
mod sonic_swap_types;

const SONIC_ID: &str = "3xwpq-ziaaa-aaaah-qcn4a-cai";
const ICP_SWAP_ID: &str = "k37c6-riaaa-aaaag-qcyza-cai";

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
                            && mapped_token.fee != 0
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
    // There might be somce tokens that are not valid for somce reasons
    // 1. Tokens wasm was removed and canister is not functional
    // 2. Token casniter ran out of cycles
    // 3. Tokens was a scam And the casniter is removed
    // This function checks if the tokens casniter is still working by simply calling the icrc1_decimals() and if cansiter
    // Returns a value, it means the token casniter is still oprating

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
            IcpTokenType::Other(_) => Ok(0),
        }
    }
}
