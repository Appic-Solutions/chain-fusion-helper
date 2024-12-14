use std::collections::HashSet;

use crate::{
    icp_tokens_service::TokenService,
    logs::INFO,
    state::{mutate_state, IcpToken},
};
use ic_canister_log::log;
pub async fn update_icp_tokens() {
    log!(
        INFO,
        "[Update Icp Tokens] Calling Sonic and Icp swap casniters to get tokens list",
    );
    let icp_swap_tokens = TokenService::new().get_icp_swap_tokens().await;
    let sonic_swap_tokens = TokenService::new().get_sonic_tokens().await;

    // Combine vectors and remove duplicates based on ledger_id
    let unique_tokens: HashSet<_> = icp_swap_tokens
        .into_iter()
        .chain(sonic_swap_tokens.into_iter())
        .collect();

    // Record new icp tokens
    // If token already exsits, it will stay as the same
    mutate_state(|s| {
        unique_tokens.into_iter().for_each(|token| {
            log!(INFO, "[Update Icp Tokens] Updating token {:?}", token);
            s.record_icp_token(token.ledger_id, token)
        });
    });
}

#[cfg(test)]
mod tests {
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
        };

        let token2 = IcpToken {
            ledger_id: Principal::from_text("5573k-xaaaa-aaaak-aacnq-cai").unwrap(), // Same ledger_id as token1
            name: String::from("TokenB"),
            decimals: 18,
            symbol: String::from("TKB"),
            token_type: IcpTokenType::DIP20,
        };

        let token3 = IcpToken {
            ledger_id: Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap(),
            name: String::from("TokenC"),
            decimals: 6,
            symbol: String::from("TKC"),
            token_type: IcpTokenType::Other("Custom".into()),
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
            },
            IcpToken {
                ledger_id: Principal::from_text("6fvyi-faaaa-aaaam-qbiga-cai").unwrap(),
                name: String::from("TokenB"),
                decimals: 18,
                symbol: String::from("TKB"),
                token_type: IcpTokenType::DIP20,
            },
        ];

        let vec2 = vec![
            IcpToken {
                ledger_id: Principal::from_text("6fvyi-faaaa-aaaam-qbiga-cai").unwrap(), // Duplicate
                name: String::from("AnotherTokenB"),
                decimals: 18,
                symbol: String::from("TKB2"),
                token_type: IcpTokenType::DIP20,
            },
            IcpToken {
                ledger_id: Principal::from_text("sr5fw-zqaaa-aaaak-qig5q-cai").unwrap(),
                name: String::from("TokenC"),
                decimals: 6,
                symbol: String::from("TKC"),
                token_type: IcpTokenType::Other("Custom".into()),
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
            },
            IcpToken {
                ledger_id: Principal::from_text("dikjh-xaaaa-aaaak-afnba-cai").unwrap(), // Duplicate
                name: String::from("AnotherTokenA"),
                decimals: 8,
                symbol: String::from("TKA"),
                token_type: IcpTokenType::ICRC2,
            },
            IcpToken {
                ledger_id: Principal::from_text("sr5fw-zqaaa-aaaak-qig5q-cai").unwrap(),
                name: String::from("TokenB"),
                decimals: 18,
                symbol: String::from("TKB"),
                token_type: IcpTokenType::DIP20,
            },
        ];

        let unique: HashSet<_> = tokens.into_iter().collect();
        assert_eq!(unique.len(), 2); // Only two unique tokens based on ledger_id
    }
}
