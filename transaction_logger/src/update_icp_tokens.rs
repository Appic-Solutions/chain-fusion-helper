use crate::{
    guard::TimerGuard,
    icp_tokens_service::TokenService,
    logs::INFO,
    state::{mutate_state, read_state, IcpToken},
};
use candid::Principal;
use futures::future::join_all;
use ic_canister_log::log;
use std::{collections::HashSet, str::FromStr};

const VALIDATION_BATCH_SIZE: usize = 5;

pub async fn update_icp_tokens() {
    // Issue a timer guard
    let _guard = match TimerGuard::new(crate::guard::TaskType::UpdateIcpTokens) {
        Ok(guard) => guard,
        Err(_) => return,
    };

    // While upgrading icp token, it is recommended to prevent usd price
    // updates.
    let _usd_price_guard = match TimerGuard::new(crate::guard::TaskType::UpdateUsdPrice) {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let token_service = TokenService::new();

    // Fetch tokens concurrently
    let (icp_swap_tokens, sonic_swap_tokens) = (
        token_service.get_icp_swap_tokens().await,
        token_service.get_sonic_tokens().await,
    );

    let mut unique_tokens = HashSet::with_capacity(icp_swap_tokens.len() + sonic_swap_tokens.len());

    // Filter the tokens that already exist in the state
    unique_tokens.retain(|token: &IcpToken| {
        read_state(|s| s.get_icp_token_by_principal(&token.ledger_id).is_none())
    });

    // Combine vectors and deduplicate on the fly
    icp_swap_tokens
        .into_iter()
        .chain(sonic_swap_tokens)
        .for_each(|token| {
            unique_tokens.insert(token);
        });

    log!(
        INFO,
        "[Update ICP Tokens] Called Sonic and ICP swap to get tokens list, Received {} tokens",
        unique_tokens.len()
    );

    // Validate process
    let icp_tokens: Vec<IcpToken> = unique_tokens.into_iter().collect();

    let validated_token =
        validate_tokens_in_batch(&icp_tokens, VALIDATION_BATCH_SIZE, &token_service).await;

    // Record new ICP tokens
    log!(
        INFO,
        "[Update ICP Tokens] Updating tokens, adding {} tokens in total",
        validated_token.len(),
    );
    mutate_state(|s| {
        for token in validated_token {
            s.record_icp_token(token.ledger_id, token.clone());
        }
    });
}

// Runs on interval basis to update usd price of icp tokens
pub async fn update_usd_price() {
    let _guard = match TimerGuard::new(crate::guard::TaskType::UpdateUsdPrice) {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let token_service = TokenService::new();

    let icp_token_with_usd_price = token_service
        .get_icp_swap_tokens_with_usd_price()
        .await
        .expect("Failed to get icp tokens with their price, will retry in next iteration");

    icp_token_with_usd_price
        .iter()
        .filter(|token| token.priceUSD != 0_f64 && token.volumeUSD7d != 0_f64)
        .for_each(|token| {
            mutate_state(|s| {
                s.update_icp_token_usd_price(
                    Principal::from_str(&token.address).unwrap_or(Principal::anonymous()),
                    token.priceUSD.to_string(),
                );
            })
        });
}

// Runs on interval basis to remove invalid tokens
pub async fn validate_tokens() {
    // Issue a timer guard
    let _guard = match TimerGuard::new(crate::guard::TaskType::RemoveInvalidTokens) {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let tokens_service = TokenService::new();

    // Get all tokens from state
    let tokens = read_state(|s| s.get_icp_tokens());

    let mut valid_tokens = 0;

    for token in tokens.iter() {
        let is_valid = tokens_service
            .validate_token(token.ledger_id, &token.token_type)
            .await
            .is_ok();

        if is_valid {
            valid_tokens += 1;
        } else {
            log!(
                INFO,
                "[Validate Tokens] Token with ledger_id {:?} is invalid and will be removed",
                token.ledger_id
            );
            mutate_state(|s| s.remove_icp_token(&token.ledger_id))
        }
    }

    log!(
        INFO,
        "[Validate Tokens] Validation complete. Remaining tokens: {}, removing {}",
        valid_tokens,
        tokens.len() - valid_tokens
    );
}

async fn validate_tokens_in_batch<'a>(
    icp_tokens: &'a [IcpToken], // Borrow tokens as a slice to avoid ownership transfer
    batch_size: usize,
    token_service: &TokenService,
) -> Vec<&'a IcpToken> {
    // Return references to the valid tokens
    let mut valid_tokens = Vec::new();

    // Chunk tokens into batches of `batch_size`
    for batch in icp_tokens.chunks(batch_size) {
        // Create futures for the current batch
        let futures = batch.iter().map(|token| {
            async move {
                match token_service
                    .validate_token(token.ledger_id, &token.token_type)
                    .await
                {
                    Ok(_) => {
                        log!(
                            INFO,
                            "[Validate Tokens] token {}, is valid",
                            token.ledger_id.to_string(),
                        );
                        Some(token)
                    } // Token is valid
                    Err(e) => {
                        log!(
                            INFO,
                            "[Validate Tokens] Failed to validate token: {}, Error: {:?}",
                            token.ledger_id.to_string(),
                            e
                        );
                        None // Token is invalid
                    }
                }
            }
        });

        // Execute all futures in the batch concurrently
        let results: Vec<Option<&IcpToken>> = join_all(futures).await;

        // Collect valid tokens from the results
        valid_tokens.extend(results.into_iter().flatten());
    }

    valid_tokens
}

#[cfg(test)]
mod tests {
    use crate::numeric::Erc20TokenAmount;
    use crate::state::IcpToken;
    use crate::state::IcpTokenType;

    use super::*;
    use candid::Principal;

    #[test]
    fn test_icp_token_equality() {
        let token1 = IcpToken {
            ledger_id: Principal::from_text("5573k-xaaaa-aaaak-aacnq-cai").unwrap(),
            name: String::from("TokenA"),
            decimals: 8,
            symbol: String::from("TKA"),
            token_type: IcpTokenType::ICRC2,
            fee: Erc20TokenAmount::from(500_u64),
            rank: Some(1),
            usd_price: "0".to_string(),
            logo: "".to_string(),
        };

        let token2 = IcpToken {
            ledger_id: Principal::from_text("5573k-xaaaa-aaaak-aacnq-cai").unwrap(), // Same ledger_id as token1
            name: String::from("TokenB"),
            decimals: 18,
            symbol: String::from("TKB"),
            token_type: IcpTokenType::DIP20,
            fee: Erc20TokenAmount::from(500_u64),
            rank: None,
            usd_price: "0".to_string(),
            logo: "".to_string(),
        };

        let token3 = IcpToken {
            ledger_id: Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap(),
            name: String::from("TokenC"),
            decimals: 6,
            symbol: String::from("TKC"),
            token_type: IcpTokenType::Other("Custom".into()),
            fee: Erc20TokenAmount::from(500_u64),
            rank: Some(2),
            usd_price: "0".to_string(),
            logo: "".to_string(),
        };

        assert_eq!(token1, token2); // Same ledger_id should mean equality
        assert_ne!(token1, token3); // Different ledger_id should mean inequality
    }

    #[test]
    fn test_remove_duplicates() {
        let vec1 = vec![
            IcpToken {
                ledger_id: Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap(),
                name: String::from("TokenA"),
                decimals: 8,
                symbol: String::from("TKA"),
                token_type: IcpTokenType::ICRC1,
                fee: Erc20TokenAmount::from(500_u64),
                rank: Some(3),
                usd_price: "0".to_string(),
                logo: "".to_string(),
            },
            IcpToken {
                ledger_id: Principal::from_text("6fvyi-faaaa-aaaam-qbiga-cai").unwrap(),
                name: String::from("TokenB"),
                decimals: 18,
                symbol: String::from("TKB"),
                token_type: IcpTokenType::DIP20,
                fee: Erc20TokenAmount::from(500_u64),
                rank: Some(2),
                usd_price: "0".to_string(),
                logo: "".to_string(),
            },
        ];

        let vec2 = vec![
            IcpToken {
                ledger_id: Principal::from_text("6fvyi-faaaa-aaaam-qbiga-cai").unwrap(),
                name: String::from("AnotherTokenB"),
                decimals: 18,
                symbol: String::from("TKB2"),
                token_type: IcpTokenType::DIP20,
                fee: Erc20TokenAmount::from(500_u64),
                rank: None,
                usd_price: "0".to_string(),
                logo: "".to_string(),
            },
            IcpToken {
                ledger_id: Principal::from_text("sr5fw-zqaaa-aaaak-qig5q-cai").unwrap(),
                name: String::from("TokenC"),
                decimals: 6,
                symbol: String::from("TKC"),
                token_type: IcpTokenType::Other("Custom".into()),
                fee: Erc20TokenAmount::from(500_u64),
                rank: Some(1),
                usd_price: "0".to_string(),
                logo: "".to_string(),
            },
        ];

        let combined: HashSet<_> = vec1.into_iter().chain(vec2.into_iter()).collect();
        let unique_tokens: Vec<IcpToken> = combined.into_iter().collect();

        assert_eq!(unique_tokens.len(), 3); // Should contain 3 unique tokens
        assert!(unique_tokens
            .iter()
            .any(|t| t.ledger_id == Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap()));
        assert!(unique_tokens
            .iter()
            .any(|t| t.ledger_id == Principal::from_text("6fvyi-faaaa-aaaam-qbiga-cai").unwrap()));
        assert!(unique_tokens
            .iter()
            .any(|t| t.ledger_id == Principal::from_text("sr5fw-zqaaa-aaaak-qig5q-cai").unwrap()));
    }

    #[test]
    fn test_hash_set_removal() {
        let tokens = vec![
            IcpToken {
                ledger_id: Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap(),
                name: String::from("TokenA"),
                decimals: 8,
                symbol: String::from("TKA"),
                token_type: IcpTokenType::ICRC1,
                fee: Erc20TokenAmount::from(500_u64),
                rank: Some(2),
                usd_price: "0".to_string(),
                logo: "".to_string(),
            },
            IcpToken {
                ledger_id: Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap(), // Duplicate
                name: String::from("AnotherTokenA"),
                decimals: 8,
                symbol: String::from("TKA"),
                token_type: IcpTokenType::ICRC2,
                fee: Erc20TokenAmount::from(500_u64),
                rank: None,
                usd_price: "0".to_string(),
                logo: "".to_string(),
            },
            IcpToken {
                ledger_id: Principal::from_text("sr5fw-zqaaa-aaaak-qig5q-cai").unwrap(),
                name: String::from("TokenB"),
                decimals: 18,
                symbol: String::from("TKB"),
                token_type: IcpTokenType::DIP20,
                fee: Erc20TokenAmount::from(500_u64),
                rank: Some(2),
                usd_price: "0".to_string(),
                logo: "".to_string(),
            },
        ];

        let unique: HashSet<_> = tokens.into_iter().collect();
        assert_eq!(unique.len(), 2); // Only two unique tokens based on ledger_id
    }
}
