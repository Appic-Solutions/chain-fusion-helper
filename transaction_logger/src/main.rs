use std::io::Chain;

use ic_canister_log::log;
use ic_cdk::{init, post_upgrade, query, update};
use ic_cdk_timers;
use transaction_logger::endpoints::{
    AddEvmToIcpTx, AddEvmToIcpTxError, AddIcpToEvmTx, AddIcpToEvmTxError,
};
use transaction_logger::lifecycle;
use transaction_logger::state::{
    mutate_state, nat_to_u64, read_state, ChainId, Erc20Identifier, EvmToIcpStatus, EvmToIcpTx,
    EvmToIcpTxIdentifier, IcpToEvmIdentifier, IcpToEvmStatus, IcpToEvmTx,
};
use transaction_logger::{
    endpoints::LoggerArgs,
    logs::INFO,
    remove_unverified_tx::remove_unverified_tx,
    scrape_events::scrape_events,
    state::{init_state, State},
    update_token_pairs::update_token_pairs,
    CHECK_NEW_ICRC_TWIN_TOKENS, REMOVE_UNVERIFIED_TX_INTERVAL, SCRAPE_EVENTS_INTERVAL,
};
// Setup timers
fn setup_timers() {
    // Start scraping events.
    ic_cdk_timers::set_timer_interval(SCRAPE_EVENTS_INTERVAL, || ic_cdk::spawn(scrape_events()));

    // Remove unverified transactions
    ic_cdk_timers::set_timer_interval(REMOVE_UNVERIFIED_TX_INTERVAL, || remove_unverified_tx());

    // Check new supported twin tokens
    ic_cdk_timers::set_timer_interval(CHECK_NEW_ICRC_TWIN_TOKENS, || {
        ic_cdk::spawn(update_token_pairs())
    });
}

#[init]
pub fn init(init_args: LoggerArgs) {
    match init_args {
        LoggerArgs::Init(init_args) => {
            log!(INFO, "[init]: initialized minter with arg: {:?}", init_args);
            let state = State::from(init_args);
            init_state(state);
        }
        LoggerArgs::Upgrade(_upgrade_arg) => {
            ic_cdk::trap("cannot init canister state with upgrade args");
        }
    }

    setup_timers();
}

#[post_upgrade]
fn post_upgrade(upgrade_args: Option<LoggerArgs>) {
    // Upgrade necessary parts if needed

    match upgrade_args {
        Some(LoggerArgs::Init(_)) => {
            ic_cdk::trap("cannot upgrade canister state with init args");
        }
        Some(LoggerArgs::Upgrade(upgrade_args)) => lifecycle::post_upgrade(Some(upgrade_args)),
        None => lifecycle::post_upgrade(None),
    }

    // Set up timers
    setup_timers();
}

// Add new icp to evm transaction
#[update]
fn new_icp_to_evm_tx(tx: AddIcpToEvmTx) -> Result<(), AddIcpToEvmTxError> {
    let tx_identifier = IcpToEvmIdentifier::from(&tx);
    let chain_id = ChainId::from(tx.chain_id.clone());

    if let true = read_state(|s| s.if_icp_to_evm_tx_exists(&tx_identifier)) {
        return Err(AddIcpToEvmTxError::TxAlreadyExsits);
    };

    if let true = read_state(|s| s.if_chain_id_exists(&chain_id)) {
        return Err(AddIcpToEvmTxError::ChinNotSupported);
    };

    let icrc_pair = read_state(|s| {
        match s.get_icrc_twin_for_erc20(
            &Erc20Identifier::new(&tx.erc20_contract_address, &chain_id),
            &tx.oprator,
        ) {
            Some(icrc_id) => Ok(icrc_id),
            None => return Err(AddIcpToEvmTxError::InvalidTokenPairs),
        }
    })?;

    mutate_state(|s| {
        s.record_new_icp_to_evm(
            tx_identifier,
            IcpToEvmTx {
                transaction_hash: None,
                native_ledger_burn_index: tx.native_ledger_burn_index,
                withdrawal_amount: tx.withdrawal_amount,
                actual_received: None,
                destination: tx.destination,
                from: tx.from,
                from_subaccount: tx.from_subaccount,
                time: nat_to_u64(tx.time),
                max_transaction_fee: Some(tx.max_transaction_fee),
                effective_gas_price: None,
                gas_used: None,
                toatal_gas_spent: None,
                erc20_ledger_burn_index: None,
                erc20_contract_address: tx.erc20_contract_address,
                icrc_ledger_id: Some(icrc_pair),
                verified: false,
                status: IcpToEvmStatus::PendingVerification,
                oprator: tx.oprator,
            },
        )
    });

    Ok(())
}

// Add new evm to icp transaction
#[update]
fn new_evm_to_icp_tx(tx: AddEvmToIcpTx) -> Result<(), AddEvmToIcpTxError> {
    let tx_identifier = EvmToIcpTxIdentifier::from(&tx);
    let chain_id = ChainId::from(tx.chain_id.clone());

    if let true = read_state(|s| s.if_evm_to_icp_tx_exists(&tx_identifier)) {
        return Err(AddEvmToIcpTxError::TxAlreadyExsits);
    };

    if let true = read_state(|s| s.if_chain_id_exists(&chain_id)) {
        return Err(AddEvmToIcpTxError::ChinNotSupported);
    };

    let icrc_pair = read_state(|s| {
        match s.get_icrc_twin_for_erc20(
            &Erc20Identifier::new(&tx.erc20_contract_address, &chain_id),
            &tx.oprator,
        ) {
            Some(icrc_id) => Ok(icrc_id),
            None => return Err(AddEvmToIcpTxError::InvalidTokenPairs),
        }
    })?;

    mutate_state(|s| {
        s.record_new_evm_to_icp(
            tx_identifier,
            EvmToIcpTx {
                transaction_hash: tx.transaction_hash,
                actual_received: None,
                time: ic_cdk::api::time(),
                erc20_contract_address: tx.erc20_contract_address,
                icrc_ledger_id: Some(icrc_pair),
                verified: false,
                status: EvmToIcpStatus::PendingVerification,
                oprator: tx.oprator,
                from_address: tx.from_address,
                value: tx.value,
                block_number: None,
                principal: tx.principal,
                subaccount: tx.subaccount,
                chain_id,
                total_gas_spent: Some(tx.total_gas_spent),
            },
        )
    });

    Ok(())
}

fn main() {}
