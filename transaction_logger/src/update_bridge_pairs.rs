use candid::Principal;
use ic_canister_log::log;

use crate::{
    guard::TimerGuard,
    ledger_manager_client::LsClient,
    logs::{DEBUG, INFO},
    state::{
        mutate_state,
        types::{BridgePair, Erc20Identifier, MinterKey, Operator},
    },
};

pub const LEDGER_SUITE_ORCHESTRATOR_ID: &str = "vxkom-oyaaa-aaaar-qafda-cai";
pub const APPIC_LEDGER_MANAGER_ID: &str = "kmcdp-4yaaa-aaaag-ats3q-cai";

/// Checks twin tokens supported by ledger_suite_orchestrator and ledger_suite_manager on an interval basis.
/// If there are new twin tokens, they are added to the state.
pub async fn update_bridge_pairs() {
    // Issue a timer guard
    let _guard = match TimerGuard::new(crate::guard::TaskType::UpdateBridgePairs) {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let managers = [
        (APPIC_LEDGER_MANAGER_ID, Operator::AppicMinter, "Appic LSM"),
        (
            LEDGER_SUITE_ORCHESTRATOR_ID,
            Operator::DfinityCkEthMinter,
            "Dfinity LSO",
        ),
    ];

    for (manager_id, operator, source_name) in managers {
        let client = LsClient::new(Principal::from_text(manager_id).unwrap(), operator.clone());

        log!(
            INFO,
            "[Scrape new twin tokens] Start scraping new twin tokens from {}",
            source_name
        );

        match client.get_erc20_list().await {
            Ok(bridge_pairs) => {
                process_bridge_pairs(bridge_pairs.get_bridge_pairs_iter(), operator, source_name)
            }
            Err(err) => {
                log!(
                    DEBUG,
                    "[Scrape new twin tokens] Failed scraping {}: {:?}",
                    source_name,
                    err
                );
            }
        }
    }
}

// Processes bridge pairs
fn process_bridge_pairs<I>(bridge_pairs: I, operator: Operator, source_name: &str)
where
    I: Iterator<Item = (Erc20Identifier, candid::Principal)>,
{
    mutate_state(|state| {
        for (erc20_identifier, principal_id) in bridge_pairs {
            if let Some(evm_token) = state.get_evm_token_by_identifier(&erc20_identifier) {
                if let Some(icp_token) = state.get_icp_token_by_principal(&principal_id) {
                    let chain_id = erc20_identifier.chain_id();
                    let minter_key = MinterKey(chain_id, operator);

                    // enable minter
                    state.enable_minter(&minter_key);

                    let bridge_pair = BridgePair {
                        icp_token,
                        evm_token,
                    };
                    log!(
                        INFO,
                        "[Scrape new bridge pairs] Recording new bridge pair {:?} from {}",
                        erc20_identifier,
                        source_name
                    );
                    match operator {
                        Operator::DfinityCkEthMinter => {
                            state
                                .supported_ckerc20_tokens
                                .insert(erc20_identifier, bridge_pair);
                        }
                        Operator::AppicMinter => {
                            state
                                .supported_twin_appic_tokens
                                .insert(erc20_identifier, bridge_pair);
                        }
                    }
                }
            }
        }
    });
}
