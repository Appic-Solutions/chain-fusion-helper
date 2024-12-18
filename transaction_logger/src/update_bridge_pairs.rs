use candid::Principal;
use ic_canister_log::log;

use crate::{
    guard::TimerGuard,
    ledger_manager_client::LsClient,
    logs::{DEBUG, INFO},
    state::{mutate_state, BridgePair},
};

const LEDGER_SUITE_ORCHESTRATOR_ID: &str = "vxkom-oyaaa-aaaar-qafda-cai";
const APPIC_LEDGER_MANAGER_ID: &str = "kmcdp-4yaaa-aaaag-ats3q-cai";

/// Checks twin tokens supported by ledger_suite_orchestrator and ledger_suite_manager on an interval basis.
/// If there are new twin tokens, they are added to the state.
pub async fn update_bridge_pairs() {
    // Issue a timer gaurd
    let _gaurd = match TimerGuard::new(crate::guard::TaskType::UpdateBridgePairs) {
        Ok(gaurd) => gaurd,
        Err(_) => return,
    };

    let managers = [
        (
            APPIC_LEDGER_MANAGER_ID,
            crate::state::Operator::AppicMinter,
            "Appic LSM",
        ),
        (
            LEDGER_SUITE_ORCHESTRATOR_ID,
            crate::state::Operator::DfinityCkEthMinter,
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

/// Processes bridge pairs, checking if they exist and adding them to the state if they do not.
/// Processes bridge pairs, checking if they exist and adding them to the state if they do not.
fn process_bridge_pairs<I>(bridge_pairs: I, operator: crate::state::Operator, source_name: &str)
where
    I: Iterator<Item = (crate::state::Erc20Identifier, candid::Principal)>,
{
    mutate_state(|state| {
        for (erc20_identifier, principal_id) in bridge_pairs {
            if state
                .get_icrc_twin_for_erc20(&erc20_identifier, &operator)
                .is_none()
            {
                if let Some(evm_token) = state.get_evm_token_by_identifier(&erc20_identifier) {
                    if let Some(icp_token) = state.get_icp_token_by_principal(&principal_id) {
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
                            crate::state::Operator::DfinityCkEthMinter => {
                                state
                                    .supported_ckerc20_tokens
                                    .insert(erc20_identifier, bridge_pair);
                            }
                            crate::state::Operator::AppicMinter => {
                                state
                                    .supported_twin_appic_tokens
                                    .insert(erc20_identifier, bridge_pair);
                            }
                        }
                    }
                }
            }
        }
    });
}
