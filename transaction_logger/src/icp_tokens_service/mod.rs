// Token service updates list of available tokens on icp on a daily basis
// Tokens are fetched from icpswap token list canister and sonic swap

use candid::Principal;
use icp_swap_token_type::TokensListResult;
use sonic_swap_types::TokenInfoWithType;

use crate::{
    minter_clinet::{IcRunTime, Runtime},
    state::IcpToken,
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
        let sonic_tokens: Vec<TokenInfoWithType> = self
            .runtime
            .call_canister(
                Principal::from_text(SONIC_ID).unwrap(),
                "getSupportedTokenList",
                (),
            )
            .await
            .unwrap();

        // Map and filter tokens into Valid IcpToken
        sonic_tokens
            .into_iter()
            .map(|token| IcpToken::from(token))
            .filter(|mapped_token| mapped_token.ledger_id != Principal::anonymous())
            .collect()
    }

    pub async fn get_icp_swap_tokens(&self) -> Vec<IcpToken> {
        // Get all icp swap supported tokens

        let icp_swap_token: TokensListResult = self
            .runtime
            .call_canister(Principal::from_text(ICP_SWAP_ID).unwrap(), "getList", ())
            .await
            .unwrap();

        // Map and filter tokens into Valid IcpToken
        match icp_swap_token {
            TokensListResult::Ok(tokens) => tokens
                .into_iter()
                .map(|token| IcpToken::from(token))
                .filter(|mapped_token| mapped_token.ledger_id != Principal::anonymous())
                .collect(),
            TokensListResult::Err(_) => vec![],
        }
    }
}
