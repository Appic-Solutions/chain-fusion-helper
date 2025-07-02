use ic_canister_log::log;
use serde_json;

const ETHEREUM_TOKENS: &str = include_str!("../../evm_tokens/eth_tokens.json");
const ARBITRUM_TOKENS: &str = include_str!("../../evm_tokens/arbitrum_tokens.json");
const AVALANCHE_TOKENS: &str = include_str!("../../evm_tokens/avalanche_tokens.json");
const BASE_TOKENS: &str = include_str!("../../evm_tokens/base_tokens.json");
const BSC_TOKENS: &str = include_str!("../../evm_tokens/bsc_tokens.json");
const FANTOM_TOKENS: &str = include_str!("../../evm_tokens/fantom_tokens.json");
const OPTIMISM_TOKENS: &str = include_str!("../../evm_tokens/optimism_tokens.json");
const POLYGON_TOKENS: &str = include_str!("../../evm_tokens/polygon_tokens.json");

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
        deserialize_all_tokens()
            .into_iter()
            .for_each(|token| s.record_evm_token(Erc20Identifier::from(&token), token))
    });
}

pub fn deserialize_all_tokens() -> Vec<EvmToken> {
    // Parse each JSON into a Vec<Token>
    let mut all_tokens = Vec::new();

    all_tokens.extend(deserialize_json_into_evm_token(ETHEREUM_TOKENS));
    all_tokens.extend(deserialize_json_into_evm_token(ARBITRUM_TOKENS));
    all_tokens.extend(deserialize_json_into_evm_token(AVALANCHE_TOKENS));
    all_tokens.extend(deserialize_json_into_evm_token(BASE_TOKENS));
    all_tokens.extend(deserialize_json_into_evm_token(BSC_TOKENS));
    all_tokens.extend(deserialize_json_into_evm_token(FANTOM_TOKENS));
    all_tokens.extend(deserialize_json_into_evm_token(OPTIMISM_TOKENS));
    all_tokens.extend(deserialize_json_into_evm_token(POLYGON_TOKENS));

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
            deserialize_all_tokens, deserialize_json_into_evm_token, ETHEREUM_TOKENS,
        },
        scrape_events::NATIVE_ERC20_ADDRESS,
        state::types::{ChainId, EvmToken},
    };

    #[test]
    fn should_deserialize_json_into_tokens() {
        let ethereum_token = EvmToken {
            chain_id: ChainId(1),
            erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
            name: "Ethereum".to_string(),
            decimals: 18,
            symbol: "ETH".to_string().to_string(),
            logo:"https://raw.githubusercontent.com/trustwallet/assets/refs/heads/master/blockchains/ethereum/info/logo.png".to_string(),
            is_wrapped_icrc:false,
        };

        assert_eq!(
            deserialize_json_into_evm_token(ETHEREUM_TOKENS)[0],
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

        assert_eq!(filtered_native_token_list.len(), 8);

        let expected_native_tokens = [
            EvmToken {
                chain_id: ChainId(1),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Ethereum".to_string(),
                decimals: 18,
                symbol: "ETH".to_string(),
                logo: "https://github.com/trustwallet/assets/blob/master/blockchains/ethereum/info/logo.png".to_string(),
                            is_wrapped_icrc:false,

            },
            EvmToken {
                chain_id: ChainId(42161),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Arbitrum One".to_string(),
                decimals: 18,
                symbol: "ETH".to_string(),
                logo: "https://github.com/trustwallet/assets/blob/master/blockchains/arbitrum/info/logo.png".to_string(),
                            is_wrapped_icrc:false,

            },
            EvmToken {
                chain_id: ChainId(43114),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Avalanche".to_string(),
                decimals: 18,
                symbol: "AVAX".to_string(),
                logo: "https://github.com/trustwallet/assets/blob/master/blockchains/avalanchec/info/logo.png".to_string(),
                            is_wrapped_icrc:false,

            },
            EvmToken {
                chain_id: ChainId(8453),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Base".to_string(),
                decimals: 18,
                symbol: "ETH".to_string(),
                logo: "https://github.com/trustwallet/assets/blob/master/blockchains/base/info/logo.png".to_string(),
                            is_wrapped_icrc:false,

            },
            EvmToken {
                chain_id: ChainId(56),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Binance Smart Chain".to_string(),
                decimals: 18,
                symbol: "BNB".to_string(),
                logo: "https://github.com/trustwallet/assets/blob/master/blockchains/smartchain/info/logo.png".to_string(),
                            is_wrapped_icrc:false,

            },
            EvmToken {
                chain_id: ChainId(250),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Fantom".to_string(),
                decimals: 18,
                symbol: "FTM".to_string(),
                logo: "https://raw.githubusercontent.com/trustwallet/assets/refs/heads/master/blockchains/fantom/info/logo.png".to_string(),
                            is_wrapped_icrc:false,

            },
            EvmToken {
                chain_id: ChainId(10),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Optimism".to_string(),
                decimals: 18,
                symbol: "ETH".to_string(),
                logo: "https://raw.githubusercontent.com/trustwallet/assets/refs/heads/master/blockchains/optimism/info/logo.png".to_string(),
                            is_wrapped_icrc:false,

            },
            EvmToken {
                chain_id: ChainId(137),
                erc20_contract_address: Address::from_str(NATIVE_ERC20_ADDRESS).unwrap(),
                name: "Polygon".to_string(),
                decimals: 18,
                symbol: "POL".to_string(),
                logo: "https://raw.githubusercontent.com/trustwallet/assets/refs/heads/master/blockchains/polygon/info/logo.png".to_string(),
                            is_wrapped_icrc:false,

            },
        ];

        assert_eq!(filtered_native_token_list, expected_native_tokens);
    }
}
