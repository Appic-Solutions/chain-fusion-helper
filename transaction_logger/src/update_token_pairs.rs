use candid::Principal;
use ic_canister_log::log;

use crate::{
    ledger_manager_client::LsClient,
    logs::{DEBUG, INFO},
    state::mutate_state,
};

const LEDGER_SUITE_ORCHESTRATOR_ID: &str = "vxkom-oyaaa-aaaar-qafda-cai";

const APPIC_LEDGER_MANAGER_ID: &str = "kmcdp-4yaaa-aaaag-ats3q-cai";

// Checks twin tokens supported by leder_suite_orchestrator and ledger_suite_manager on an interval base,
// If there is a new twin token, then it will be added to the state.
pub async fn update_token_pairs() {
    let appic_ledger_manager_clinet = LsClient::new(
        Principal::from_text(APPIC_LEDGER_MANAGER_ID).unwrap(),
        crate::state::Operator::AppicMinter,
    );

    log!(
        INFO,
        "[Scrape new twin tokens] Start scraping new twin tokens from appic lsm",
    );
    let appic_token_paris = appic_ledger_manager_clinet.get_erc20_list().await;

    match appic_token_paris {
        Ok(token_pairs) => {
            for (erc20_identifier, principal_id) in token_pairs.get_token_pairs_iter() {
                mutate_state(|s| {
                    // Check if token exsits
                    if s.get_icrc_twin_for_erc20(
                        &erc20_identifier,
                        &crate::state::Operator::AppicMinter,
                    )
                    .is_none()
                    {
                        log!(
                         INFO,
                            "[Scrape new twin tokens] Adding new twin token to state with identifier {:?} and principal {:?} from Appic LSM",
                             erc20_identifier,
                            principal_id.to_text()
                        );
                        s.supported_twin_appic_tokens
                            .insert(erc20_identifier, principal_id);
                    }
                })
            }
        }
        Err(err) => {
            log!(
                DEBUG,
                "[Scrape new twin tokens] Failed scraping appic lsm reason {:?}",
                err
            );
        }
    }

    let dfinity_ledger_manager_clinet = LsClient::new(
        Principal::from_text(LEDGER_SUITE_ORCHESTRATOR_ID).unwrap(),
        crate::state::Operator::DfinityCkEthMinter,
    );

    log!(
        INFO,
        "[Scrape new twin tokens] Start scraping new twin tokens from dfinity lso",
    );

    let dfinity_token_paris = dfinity_ledger_manager_clinet.get_erc20_list().await;

    match dfinity_token_paris {
        Ok(token_pairs) => {
            for (erc20_identifier, principal_id) in token_pairs.get_token_pairs_iter() {
                mutate_state(|s| {
                    // Check if token exsits
                    if s.get_icrc_twin_for_erc20(
                        &erc20_identifier,
                        &crate::state::Operator::DfinityCkEthMinter,
                    )
                    .is_none()
                    {
                        log!(
                            INFO,
                            "[Scrape new twin tokens] Adding new twin token to state with identifier {:?} and principal {:?} from Dfinity LSO",
                            erc20_identifier,
                            principal_id.to_text()
                        );
                        s.supported_ckerc20_tokens
                            .insert(erc20_identifier, principal_id);
                    }
                })
            }
        }
        Err(err) => {
            log!(
                DEBUG,
                "[Scrape new twin tokens] Failed scraping dfinity lso reason {:?}",
                err
            );
        }
    }
}
