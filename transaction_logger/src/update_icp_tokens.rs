use futures::future::join_all;
use std::collections::HashSet;

use crate::{
    guard::TimerGuard,
    icp_tokens_service::TokenService,
    logs::INFO,
    state::{mutate_state, read_state, IcpToken},
};
use ic_canister_log::log;

pub async fn update_icp_tokens() {
    // Issue a timer gaurd
    let _gaurd = match TimerGuard::new(crate::guard::TaskType::UpdateIcpTokens) {
        Ok(gaurd) => gaurd,
        Err(_) => return,
    };

    let token_service = TokenService::new();

    // Fetch tokens concurrently
    let (icp_swap_tokens, sonic_swap_tokens) = (
        token_service.get_icp_swap_tokens().await,
        token_service.get_sonic_tokens().await,
    );

    let mut unique_tokens = HashSet::with_capacity(icp_swap_tokens.len() + sonic_swap_tokens.len());

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

    // Validate tokens
    // Create a vector of futures for validation
    let futures = unique_tokens.into_iter().map(|token| {
        // Spawn an async block for the validation call
        async {
            let result = match token_service
                .validate_token(token.ledger_id, &token.token_type)
                .await
            {
                Ok(_) => Some(token), // Token is valid
                Err(_) => None,       // Token is invalid
            };
            result
        }
    });
    let valid_tokens: Vec<IcpToken> = join_all(futures)
        .await
        .into_iter()
        .filter_map(|token| token)
        .collect();

    // Record new ICP tokens
    log!(
        INFO,
        "[Update ICP Tokens] Updating tokens, adding {} tokens in total",
        valid_tokens.len(),
    );
    mutate_state(|s| {
        for token in valid_tokens {
            s.record_icp_token(token.ledger_id, token);
        }
    });
}

// Runs every week to remove invalid tokens
pub async fn validate_tokens() {
    // Issue a timer gaurd
    let _gaurd = match TimerGuard::new(crate::guard::TaskType::RemoveInvalidTokens) {
        Ok(gaurd) => gaurd,
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

#[cfg(test)]
mod tests {
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
            fee: 500,
            rank: Some(1),
        };

        let token2 = IcpToken {
            ledger_id: Principal::from_text("5573k-xaaaa-aaaak-aacnq-cai").unwrap(), // Same ledger_id as token1
            name: String::from("TokenB"),
            decimals: 18,
            symbol: String::from("TKB"),
            token_type: IcpTokenType::DIP20,
            fee: 500,
            rank: None,
        };

        let token3 = IcpToken {
            ledger_id: Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap(),
            name: String::from("TokenC"),
            decimals: 6,
            symbol: String::from("TKC"),
            token_type: IcpTokenType::Other("Custom".into()),
            fee: 500,
            rank: Some(2),
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
                fee: 500,
                rank: Some(3),
            },
            IcpToken {
                ledger_id: Principal::from_text("6fvyi-faaaa-aaaam-qbiga-cai").unwrap(),
                name: String::from("TokenB"),
                decimals: 18,
                symbol: String::from("TKB"),
                token_type: IcpTokenType::DIP20,
                fee: 500,
                rank: Some(2),
            },
        ];

        let vec2 = vec![
            IcpToken {
                ledger_id: Principal::from_text("6fvyi-faaaa-aaaam-qbiga-cai").unwrap(), // Duplicate
                name: String::from("AnotherTokenB"),
                decimals: 18,
                symbol: String::from("TKB2"),
                token_type: IcpTokenType::DIP20,
                fee: 500,
                rank: None,
            },
            IcpToken {
                ledger_id: Principal::from_text("sr5fw-zqaaa-aaaak-qig5q-cai").unwrap(),
                name: String::from("TokenC"),
                decimals: 6,
                symbol: String::from("TKC"),
                token_type: IcpTokenType::Other("Custom".into()),
                fee: 500,
                rank: Some(1),
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
                fee: 500,
                rank: Some(2),
            },
            IcpToken {
                ledger_id: Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap(), // Duplicate
                name: String::from("AnotherTokenA"),
                decimals: 8,
                symbol: String::from("TKA"),
                token_type: IcpTokenType::ICRC2,
                fee: 500,
                rank: None,
            },
            IcpToken {
                ledger_id: Principal::from_text("sr5fw-zqaaa-aaaak-qig5q-cai").unwrap(),
                name: String::from("TokenB"),
                decimals: 18,
                symbol: String::from("TKB"),
                token_type: IcpTokenType::DIP20,
                fee: 500,
                rank: Some(2),
            },
        ];

        let unique: HashSet<_> = tokens.into_iter().collect();
        assert_eq!(unique.len(), 2); // Only two unique tokens based on ledger_id
    }
}
