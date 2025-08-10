use ic_canister_log::log;
use serde_json;

const SUPPORTED_EVM_TOKENS: &str = include_str!("../../evm_tokens/supported_tokens.json");

use crate::{
    logs::INFO,
    state::{
        mutate_state,
        types::{Erc20Identifier, EvmToken},
    },
};

pub fn add_evm_tokens_to_state() {
    log!(
        INFO,
        "[Add EVM Tokens] Adding new EVM tokens from json files",
    );
    mutate_state(|s| {
        let tokens: Vec<(Erc20Identifier, EvmToken)> = s.evm_token_list.iter().collect();
        for token in tokens {
            s.evm_token_list.remove(&token.0);
        }

        deserialize_all_tokens()
            .into_iter()
            .for_each(|token| s.record_evm_token(Erc20Identifier::from(&token), token))
    });
}

pub fn deserialize_all_tokens() -> Vec<EvmToken> {
    // Parse each JSON into a Vec<Token>
    let mut all_tokens = Vec::new();

    all_tokens.extend(deserialize_json_into_evm_token(SUPPORTED_EVM_TOKENS));

    all_tokens
}

pub fn deserialize_json_into_evm_token(json_str: &str) -> Vec<EvmToken> {
    serde_json::from_str(json_str).unwrap()
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use crate::address::Address;

    use crate::{
        add_evm_tokens::{
            deserialize_all_tokens, deserialize_json_into_evm_token, SUPPORTED_EVM_TOKENS,
        },
        scrape_events::NATIVE_ERC20_ADDRESS,
        state::types::{ChainId, EvmToken},
    };

    #[test]
    fn should_deserialize_json_into_tokens() {
        let ethereum_token = EvmToken {
            chain_id: ChainId(56),
            erc20_contract_address: Address::from_str("0x4338665cbb7b2485a8855a139b75d5e34ab0db94")
                .unwrap(),
            name: "Litecoin".to_string(),
            decimals: 18,
            symbol: "LTC".to_string().to_string(),
            logo: "https://s2.coinmarketcap.com/static/img/coins/64x64/2.png".to_string(),
            is_wrapped_icrc: false,
            cmc_id: Some(2),
            usd_price: None,
            volume_usd_24h: None,
        };

        assert_eq!(
            deserialize_json_into_evm_token(SUPPORTED_EVM_TOKENS)[0],
            ethereum_token
        )
    }

    #[test]
    fn should_deserialize_all_tokens() {
        // There should be 8 native tokens in deserialized tokens.
        let filtered_native_token_list: Vec<EvmToken> = deserialize_all_tokens()
            .into_iter()
            .filter(|token| {
                token.erc20_contract_address == Address::from_str(NATIVE_ERC20_ADDRESS).unwrap()
            })
            .collect();

        assert_eq!(filtered_native_token_list.len(), 3);

        let expected_native_tokens = [
            EvmToken {
                chain_id: ChainId(1),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Ethereum".to_string(),
                decimals: 18,
                symbol: "ETH".to_string(),
                logo: "https://s2.coinmarketcap.com/static/img/coins/64x64/1027.png".to_string(),
                is_wrapped_icrc: false,
                cmc_id: Some(1027),
                usd_price: None,
                volume_usd_24h: None,
            },
            EvmToken {
                chain_id: ChainId(8453),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Ethereum".to_string(),
                decimals: 18,
                symbol: "ETH".to_string(),
                logo: "https://s2.coinmarketcap.com/static/img/coins/64x64/1027.png".to_string(),
                is_wrapped_icrc: false,
                cmc_id: Some(1027),
                usd_price: None,
                volume_usd_24h: None,
            },
            EvmToken {
                chain_id: ChainId(56),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "BNB".to_string(),
                decimals: 18,
                symbol: "BNB".to_string(),
                logo: "https://s2.coinmarketcap.com/static/img/coins/64x64/1839.png".to_string(),
                is_wrapped_icrc: false,
                cmc_id: Some(1839),
                usd_price: None,
                volume_usd_24h: None,
            },
        ];

        assert_eq!(filtered_native_token_list, expected_native_tokens);
    }
}
