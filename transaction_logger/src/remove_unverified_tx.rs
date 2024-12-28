use ic_canister_log::log;

use crate::{
    guard::TimerGuard,
    logs::INFO,
    state::{mutate_state, read_state},
};

// If the transaction time is older than one hour and it is still unverified,
// Tx should be removed

const ONE_HOUR_IN_NS: u64 = 3_600_000_000_000;

pub fn remove_unverified_tx() {
    // Issue a timer guard
    let _guard = match TimerGuard::new(crate::guard::TaskType::RemoveUnverified) {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let all_unverified_evm_to_icp_tx = read_state(|s| s.all_unverified_evm_to_icp());

    log!(INFO, "[Remove Unverified Tx] Removing unverified tx");

    let current_time = ic_cdk::api::time();
    for (identifier, tx_time) in all_unverified_evm_to_icp_tx {
        if tx_time + ONE_HOUR_IN_NS < current_time {
            log!(
                INFO,
                "[Remove Unverified Tx] Removing unverified tx with identifier {:?}",
                identifier,
            );
            mutate_state(|s| s.remove_unverified_evm_to_icp(&identifier))
        }
    }

    let all_unverified_icp_to_evm_tx = read_state(|s| s.all_unverified_icp_to_evm());

    for (identifier, tx_time) in all_unverified_icp_to_evm_tx {
        if tx_time + ONE_HOUR_IN_NS < current_time {
            log!(
                INFO,
                "[Remove Unverified Tx] Removing unverified tx with identifier {:?} ",
                identifier,
            );
            mutate_state(|s| s.remove_unverified_icp_to_evm(&identifier))
        }
    }
}
