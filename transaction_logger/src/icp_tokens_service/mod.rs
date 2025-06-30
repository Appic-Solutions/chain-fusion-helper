// Token service updates list of available tokens on icp on a daily basis
// Tokens are fetched from icpswap token list canister and sonic swap

use std::collections::{HashMap, HashSet};

use candid::{CandidType, Principal};
use ethnum::U256;
use ic_canister_log::log;
use icp_swap_token_type::TokensListResult;
use icp_swap_usd_node_types::PublicTokenOverview;
use num_bigint::BigUint;
use serde::Deserialize;

use crate::{
    appic_dex_types::{CandidPoolId, CandidPoolState},
    logs::INFO,
    minter_client::{CallError, IcRunTime, Reason, Runtime},
    numeric::Erc20TokenAmount,
    state::{read_state, IcpToken, IcpTokenType},
};

#[derive(CandidType, Deserialize, Debug)]
pub enum MetadataValue {
    Int(candid::Int),
    Nat(candid::Nat),
    Blob(serde_bytes::ByteBuf),
    Text(String),
}

mod icp_swap_token_type;
mod icp_swap_usd_node_types;

#[cfg(test)]
pub mod tests;

const ICP_SWAP_ID: &str = "k37c6-riaaa-aaaag-qcyza-cai";
const ICP_SWAP_NODE: &str = "ggzvv-5qaaa-aaaag-qck7a-cai";
const APPIC_DEX_CANISTER_ID: &str = "nbepk-iyaaa-aaaad-qhlma-cai";

const CKUSDC_LEDGER_ID: &str = "xevnm-gaaaa-aaaar-qafnq-cai";

pub struct TokenService {
    runtime: IcRunTime,
}

impl TokenService {
    pub fn new() -> Self {
        Self {
            runtime: IcRunTime {},
        }
    }

    pub async fn get_appic_dex_tokens(&self) -> Vec<IcpToken> {
        let mut unique_tokens: HashSet<Principal> = HashSet::new();

        match self
            .runtime
            .call_canister::<(), Vec<(CandidPoolId, CandidPoolState)>>(
                Principal::from_text(APPIC_DEX_CANISTER_ID).unwrap(),
                "get_pools",
                (),
            )
            .await
        {
            Ok(pools) => {
                pools.iter().for_each(|(id, _)| {
                    unique_tokens.insert(id.token0);
                    unique_tokens.insert(id.token1);
                });
            }
            Err(e) => {
                log!(INFO, "Failed To get appic_dex tokens for {}", e);

                return vec![];
            }
        };

        // Filter the tokens that already exist in the state
        unique_tokens
            .retain(|token| read_state(|s| s.get_icp_token_by_principal(&token).is_none()));

        let mut validated_icp_tokens = Vec::new();

        for ledger_id in unique_tokens.iter() {
            match self.validate_token(*ledger_id, Some(1_u32)).await {
                Ok(token) => validated_icp_tokens.push(token),
                Err(e) => {
                    log!(
                        INFO,
                        "Failed To validate token {:?} for {}",
                        ledger_id.to_text(),
                        e
                    );
                }
            }
        }

        validated_icp_tokens
    }

    pub async fn get_icp_swap_tokens(&self) -> Vec<IcpToken> {
        // Get all icp swap supported tokens

        let mut unique_tokens: HashSet<Principal> = match self
            .runtime
            .call_canister::<(), TokensListResult>(
                Principal::from_text(ICP_SWAP_ID).unwrap(),
                "getList",
                (),
            )
            .await
        {
            Ok(tokens) => match tokens {
                // Map and filter tokens into Valid IcpToken
                TokensListResult::Ok(tokens) => {
                    HashSet::from_iter(tokens.into_iter().filter_map(|token| {
                        let ledger_id = Principal::from_text(token.canister_id).ok()?;

                        if token.standard.as_str() == "ICRC2"
                            || token.standard.as_str() == "ICRC3"
                            || token.rank < 5_u32
                        {
                            return Some(ledger_id);
                        } else {
                            None
                        }
                    }))
                }
                TokensListResult::Err(e) => {
                    log!(INFO, "Failed To get icp_swap tokens for {}", e);

                    return vec![];
                }
            },
            Err(e) => {
                // Error handling

                log!(INFO, "Failed To get icp_swap tokens for {}", e);

                return vec![];
            }
        };

        //Filter the tokens that already exist in the state
        unique_tokens
            .retain(|token| read_state(|s| s.get_icp_token_by_principal(&token).is_none()));

        let mut validated_icp_tokens = Vec::new();

        for ledger_id in unique_tokens.iter() {
            match self.validate_token(*ledger_id, Some(1_u32)).await {
                Ok(token) => validated_icp_tokens.push(token),
                Err(e) => {
                    log!(
                        INFO,
                        "Failed To validate token {:?} for {}",
                        ledger_id.to_text(),
                        e
                    );
                }
            }
        }

        validated_icp_tokens
    }

    // Validate tokens on icp
    // There might be some tokens that are not valid for some reasons
    // 1. Tokens wasm was removed and canister is not functional
    // 2. Token canister ran out of cycles
    // 3. Tokens was a scam And the canister is removed
    // This function checks if the tokens canister is still working by simply calling the icrc1_decimals() and if canister
    // Returns token logo, it means the token canister is still operating

    pub async fn validate_token(
        &self,
        ledger_id: Principal,
        rank: Option<u32>,
    ) -> Result<IcpToken, CallError> {
        match self
            .runtime
            .call_canister::<(), Vec<(String, MetadataValue)>>(ledger_id, "icrc1_metadata", ())
            .await
        {
            // If error try again.
            Ok(metadata) => {
                if let Some(icp_token) = convert_to_icp_token(ledger_id, metadata, rank).ok() {
                    return Ok(icp_token);
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

    pub async fn get_appic_dex_tokens_usd_price(&self) -> Result<Vec<(Principal, f64)>, String> {
        let ck_usdc = Principal::from_text(CKUSDC_LEDGER_ID)
            .map_err(|e| format!("Invalid ckUSDC principal: {:?}", e))?;

        let pools = self
            .runtime
            .call_canister::<(), Vec<(CandidPoolId, CandidPoolState)>>(
                Principal::from_text(APPIC_DEX_CANISTER_ID).unwrap(),
                "get_pools",
                (),
            )
            .await
            .map_err(|e| format!("Failed to fetch pools: {:?}", e))?;

        // Filter pools with ckUSDC and collect tokens to query decimals
        let mut relevant_pools: Vec<(CandidPoolId, CandidPoolState, Principal)> = Vec::new();
        let mut tokens_to_query: Vec<Principal> = Vec::new();

        for (pool_id, pool_state) in pools {
            if pool_id.token0 == ck_usdc {
                relevant_pools.push((pool_id.clone(), pool_state, pool_id.token1));
                tokens_to_query.push(pool_id.token1);
            } else if pool_id.token1 == ck_usdc {
                relevant_pools.push((pool_id.clone(), pool_state, pool_id.token0));
                tokens_to_query.push(pool_id.token0);
            }
        }

        // Add ckUSDC to tokens to query
        tokens_to_query.push(ck_usdc);

        // Query decimals and cache them
        let mut decimals_cache = HashMap::new();
        for token in tokens_to_query
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
        {
            let decimals = self
                .get_decimals(token)
                .await
                .map_err(|e| format!("Failed to get decimals for token {}: {}", token, e))?;
            decimals_cache.insert(token, decimals);
        }

        // Process each relevant pool
        let mut results = Vec::new();

        for (pool_id, pool_state, other_token) in relevant_pools {
            let usd_price = claculate_usd_price_based_on_ck_usdc(
                &pool_id,
                pool_state,
                &other_token,
                &ck_usdc,
                &decimals_cache,
            );
            // Format the price as a string with 8 decimal places
            results.push((other_token, usd_price));
        }

        Ok(results)
    }

    pub async fn get_icp_swap_tokens_with_usd_price(
        &self,
    ) -> Result<Vec<(Principal, f64)>, CallError> {
        self.runtime
            .call_canister::<(), Vec<PublicTokenOverview>>(
                Principal::from_text(ICP_SWAP_NODE).unwrap(),
                "getAllTokens",
                (),
            )
            .await
            .map(|tokens| {
                tokens
                    .into_iter()
                    .filter_map(|token| {
                        if token.priceUSD != 0_f64 && token.volumeUSD7d != 0_f64 {
                            if let Some(ledger_id) = Principal::from_text(token.address).ok() {
                                Some((ledger_id, token.priceUSD))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect()
            })
    }

    // Function to get token decimals (assuming ICRC-1 standard)
    pub async fn get_decimals(&self, token: Principal) -> Result<u8, String> {
        // Step 1: Check the state for the token
        let token_opt = read_state(|s| s.get_icp_token_by_principal(&token));

        if let Some(token) = token_opt {
            // Token found in state, return its decimals
            return Ok(token.decimals);
        }

        match self
            .runtime
            .call_canister::<(), u8>(token, "icrc1_decimals", ())
            .await
        {
            Ok(decimals) => Ok(decimals),
            Err(e) => Err(format!("Failed to get decimals: {:?}", e)),
        }
    }
}

// Function to convert metadata to IcpToken
pub fn convert_to_icp_token(
    ledger_id: Principal,
    metadata: Vec<(String, MetadataValue)>,
    rank: Option<u32>,
) -> Result<IcpToken, String> {
    let mut name = None;
    let mut decimals = None;
    let mut symbol = None;
    let mut logo = None;
    let mut fee = None;

    for (key, value) in metadata {
        match (key.as_str(), value) {
            ("icrc1:name", MetadataValue::Text(val)) => name = Some(val),
            ("icrc1:decimals", MetadataValue::Nat(val)) => {
                decimals = Some(
                    val.0
                        .try_into()
                        .map_err(|_| "Decimals out of range for u8")?,
                )
            }
            ("icrc1:symbol", MetadataValue::Text(val)) => symbol = Some(val),
            ("icrc1:logo", MetadataValue::Text(val)) => logo = Some(val),
            ("icrc1:fee", MetadataValue::Nat(val)) => {
                fee = Some(Erc20TokenAmount::try_from(val).map_err(|_| "Fee out of range for u64")?)
            }
            _ => {} // Ignore unrecognized keys
        }
    }

    Ok(IcpToken {
        ledger_id,
        name: name.ok_or("Missing icrc1:name")?,
        decimals: decimals.ok_or("Missing icrc1:decimals")?,
        symbol: symbol.ok_or("Missing icrc1:symbol")?,
        usd_price: String::from("0"), // Not provided in metadata, set as empty
        logo: logo.ok_or("Missing icrc1:logo")?,
        fee: fee.ok_or("Missing icrc1:fee")?,
        token_type: IcpTokenType::ICRC2,
        rank,
    })
}

pub fn big_uint_to_u256(biguint: BigUint) -> Result<U256, String> {
    let value_bytes = biguint.to_bytes_be();
    let mut value_u256 = [0u8; 32];
    if value_bytes.len() <= 32 {
        value_u256[32 - value_bytes.len()..].copy_from_slice(&value_bytes);
    } else {
        return Err(format!("does not fit in a U256: {}", biguint));
    }
    Ok(U256::from_be_bytes(value_u256))
}

pub fn claculate_usd_price_based_on_ck_usdc(
    pool_id: &CandidPoolId,
    pool_state: CandidPoolState,
    other_token: &Principal,
    ck_usdc_ledger_id: &Principal,
    decimals_cache: &HashMap<Principal, u8>,
) -> f64 {
    let q96 = 79228162514264337593543950336_f64;

    // Convert sqrt_price_x96 from Nat to f64
    let sqrt_price_x96 = pool_state
        .sqrt_price_x96
        .0
        .to_string()
        .parse::<f64>()
        .expect("sqrt_price_x96 should fit in a f64");

    let p: f64 = (sqrt_price_x96 / q96).powi(2_i32);

    println!("Q96: {}", q96);
    println!(" sqrt_price_x96: {}", sqrt_price_x96);
    println!(" calculated p: {}", p);

    let d_usdc = decimals_cache[ck_usdc_ledger_id] as i32;
    let d_other = decimals_cache[other_token] as i32;

    let usd_price = if &pool_id.token0 == ck_usdc_ledger_id {
        // Case 1: ckUSDC is token0, other_token is token1
        // USD price = 10^d_other / (P * 10^d_usdc)
        10.0_f64.powi(d_other) / (p * 10.0_f64.powi(d_usdc))
    } else {
        // Case 2: ckUSDC is token1, other_token is token0
        // USD price = (P * 10^d_other) / 10^d_usdc
        (p * 10.0_f64.powi(d_other)) / 10.0_f64.powi(d_usdc)
    };

    usd_price
}
