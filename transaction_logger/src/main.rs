use std::str::FromStr;
use std::time::Duration;

use candid::Principal;
use ic_canister_log::log;
use ic_cdk::{init, post_upgrade, query, update};
use ic_cdk_timers;
use ic_ethereum_types::Address;
use transaction_logger::add_evm_tokens::add_evm_tokens_to_state;
use transaction_logger::endpoints::{
    AddEvmToIcpTx, AddEvmToIcpTxError, AddIcpToEvmTx, AddIcpToEvmTxError, CandidEvmToken,
    CandidIcpToken, GetEvmTokenArgs, GetIcpTokenArgs, GetTxParams, Icrc28TrustedOriginsResponse,
    TokenPair, Transaction,
};
use transaction_logger::guard::{TaskType, TimerGuard};
use transaction_logger::lifecycle::{self, init as initialize};
use transaction_logger::state::{
    mutate_state, nat_to_erc20_amount, nat_to_ledger_burn_index, nat_to_u64, read_state, ChainId,
    Erc20Identifier, EvmToIcpStatus, EvmToIcpTx, EvmToIcpTxIdentifier, IcpToEvmIdentifier,
    IcpToEvmStatus, IcpToEvmTx,
};
use transaction_logger::update_icp_tokens::{update_icp_tokens, validate_tokens};
use transaction_logger::{
    endpoints::LoggerArgs, logs::INFO, remove_unverified_tx::remove_unverified_tx,
    scrape_events::scrape_events, update_bridge_pairs::update_bridge_pairs, REMOVE_UNVERIFIED_TX,
    SCRAPE_EVENTS, UPDATE_BRIDGE_PAIRS,
};
use transaction_logger::{REMOVE_INVALID_ICP_TOKENS, UPDATE_ICP_TOKENS};
// Setup timers
fn setup_timers() {
    // Start scraping events.
    ic_cdk_timers::set_timer_interval(SCRAPE_EVENTS, || ic_cdk::spawn(scrape_events()));

    // Remove unverified transactions
    ic_cdk_timers::set_timer_interval(REMOVE_UNVERIFIED_TX, || remove_unverified_tx());

    // Check new supported twin tokens
    ic_cdk_timers::set_timer_interval(UPDATE_BRIDGE_PAIRS, || ic_cdk::spawn(update_bridge_pairs()));

    // Update Icp token list
    ic_cdk_timers::set_timer_interval(UPDATE_ICP_TOKENS, || ic_cdk::spawn(update_icp_tokens()));

    // Remove invalid icp tokens
    ic_cdk_timers::set_timer_interval(REMOVE_INVALID_ICP_TOKENS, || {
        ic_cdk::spawn(validate_tokens())
    });
}

#[init]
pub fn init(init_args: LoggerArgs) {
    match init_args {
        LoggerArgs::Init(init_args) => {
            log!(INFO, "[init]: initialized minter with arg: {:?}", init_args);

            initialize(init_args);
        }
        LoggerArgs::Upgrade(_upgrade_arg) => {
            ic_cdk::trap("cannot init canister state with upgrade args");
        }
    }

    prepare_canister_state();

    // setup_timers();
}

fn prepare_canister_state() {
    // Add Evm tokens to state
    add_evm_tokens_to_state();

    // Get all the icp tokens and save them into state
    // Then get all bridge pairs
    ic_cdk_timers::set_timer(Duration::from_secs(0), || {
        ic_cdk::spawn(get_icp_tokens_and_bridge_pairs())
    });
}

pub async fn get_icp_tokens_and_bridge_pairs() {
    // Ensures that scraping events will be blocked until
    // All tokens are added to cansiter state
    let _guard: TimerGuard =
        TimerGuard::new(TaskType::ScrapeEvents).expect("No guard should exsist at this point");

    update_icp_tokens().await;
    update_bridge_pairs().await;
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
    let chain_id = ChainId::from(&tx.chain_id);

    if let true = read_state(|s| s.if_icp_to_evm_tx_exists(&tx_identifier)) {
        return Err(AddIcpToEvmTxError::TxAlreadyExsits);
    };

    if let true = read_state(|s| s.if_chain_id_exists(chain_id)) {
        return Err(AddIcpToEvmTxError::ChinNotSupported);
    };

    let destination =
        Address::from_str(&tx.destination).map_err(|_e| AddIcpToEvmTxError::InvalidDestination)?;

    let erc20_contract_address = Address::from_str(&tx.erc20_contract_address)
        .map_err(|_e| AddIcpToEvmTxError::InvalidTokenContract)?;

    let icrc_pair = read_state(|s| {
        match s.get_icrc_twin_for_erc20(
            &Erc20Identifier::new(&erc20_contract_address, chain_id),
            &tx.operator,
        ) {
            Some(icrc_id) => Ok(icrc_id),
            None => return Err(AddIcpToEvmTxError::InvalidTokenPairs),
        }
    })?;

    log!(INFO, "[Add New Icp to Evm Transaction] tx: {:?}", tx);
    mutate_state(|s| {
        s.record_new_icp_to_evm(
            tx_identifier,
            IcpToEvmTx {
                transaction_hash: None,
                native_ledger_burn_index: nat_to_ledger_burn_index(&tx.native_ledger_burn_index),
                withdrawal_amount: nat_to_erc20_amount(tx.withdrawal_amount),
                actual_received: None,
                destination,
                from: tx.from,
                from_subaccount: tx.from_subaccount,
                time: nat_to_u64(&tx.time),
                max_transaction_fee: Some(nat_to_erc20_amount(tx.max_transaction_fee)),
                effective_gas_price: None,
                gas_used: None,
                total_gas_spent: None,
                erc20_ledger_burn_index: None,
                erc20_contract_address,
                icrc_ledger_id: Some(icrc_pair),
                verified: false,
                status: IcpToEvmStatus::PendingVerification,
                operator: tx.operator,
                chain_id,
            },
        )
    });

    Ok(())
}

// Add new evm to icp transaction
#[update]
fn new_evm_to_icp_tx(tx: AddEvmToIcpTx) -> Result<(), AddEvmToIcpTxError> {
    let tx_identifier = EvmToIcpTxIdentifier::from(&tx);
    let chain_id = ChainId::from(&tx.chain_id);

    if let true = read_state(|s| s.if_evm_to_icp_tx_exists(&tx_identifier)) {
        return Err(AddEvmToIcpTxError::TxAlreadyExsits);
    };

    if let true = read_state(|s| s.if_chain_id_exists(chain_id)) {
        return Err(AddEvmToIcpTxError::ChinNotSupported);
    };

    let from_address =
        Address::from_str(&tx.from_address).map_err(|_e| AddEvmToIcpTxError::InvalidAddress)?;

    let erc20_contract_address = Address::from_str(&tx.erc20_contract_address)
        .map_err(|_e| AddEvmToIcpTxError::InvalidTokenContract)?;

    let icrc_pair = read_state(|s| {
        match s.get_icrc_twin_for_erc20(
            &Erc20Identifier::new(&erc20_contract_address, chain_id),
            &tx.operator,
        ) {
            Some(icrc_id) => Ok(icrc_id),
            None => return Err(AddEvmToIcpTxError::InvalidTokenPairs),
        }
    })?;

    log!(INFO, "[Add New Evm to Icp Transaction] tx: {:?}", tx);

    mutate_state(|s| {
        s.record_new_evm_to_icp(
            tx_identifier,
            EvmToIcpTx {
                transaction_hash: tx.transaction_hash,
                actual_received: None,
                time: ic_cdk::api::time(),
                erc20_contract_address,
                icrc_ledger_id: Some(icrc_pair),
                ledger_mint_index: None,
                verified: false,
                status: EvmToIcpStatus::PendingVerification,
                operator: tx.operator,
                from_address,
                value: nat_to_erc20_amount(tx.value),
                block_number: None,
                principal: tx.principal,
                subaccount: tx.subaccount,
                chain_id,
                total_gas_spent: Some(nat_to_erc20_amount(tx.total_gas_spent)),
            },
        )
    });

    Ok(())
}

#[query]
pub fn get_txs_by_address(address: String) -> Vec<Transaction> {
    let address = Address::from_str(&address).expect("Address should be valid");
    read_state(|s| s.get_transaction_for_address(address))
}

#[query]
pub fn get_txs_by_principal(principal_id: Principal) -> Vec<Transaction> {
    read_state(|s| s.get_transaction_for_principal(principal_id))
}

#[query]
pub fn get_bridge_pairs() -> Vec<TokenPair> {
    read_state(|s| s.get_suported_bridge_pairs())
}

#[query]
pub fn get_transaction(params: GetTxParams) -> Option<Transaction> {
    // Check if chain id is supported
    let chain_id = ChainId::from(&params.chain_id);
    let chain_check_result = read_state(|s| s.if_chain_id_exists(chain_id));

    if !chain_check_result {
        return None;
    }

    let search_result =
        read_state(|s| s.get_transaction_by_search_params(params.search_param, chain_id));

    search_result
}

#[query]
pub fn get_evm_token(args: GetEvmTokenArgs) -> Option<CandidEvmToken> {
    // Validate address and create identifier
    let identifier = Erc20Identifier::new(
        &Address::from_str(&args.address).expect("Wrong Address Provided"),
        ChainId::from(&args.chain_id),
    );

    // Get token from state
    let token = read_state(|s| s.get_evm_token_by_identifier(&identifier))?;

    // Return Token
    Some(CandidEvmToken::from(token))
}

#[query]
pub fn get_icp_token(args: GetIcpTokenArgs) -> Option<CandidIcpToken> {
    // Get token from state
    let token = read_state(|s| s.get_icp_token_by_principal(&args.ledger_id))?;

    // Return Token
    Some(CandidIcpToken::from(token))
}

#[query]
pub fn get_icp_tokens() -> Vec<CandidIcpToken> {
    // Get tokens from state
    let tokens = read_state(|s| s.get_icp_tokens());

    // Return Tokens
    tokens
        .into_iter()
        .map(|token| CandidIcpToken::from(token))
        .collect()
}

// list every base URL that users will authenticate to your app from
#[update]
fn icrc28_trusted_origins() -> Icrc28TrustedOriginsResponse {
    let trusted_origins = vec![
        String::from("https://dduc6-3yaaa-aaaal-ai63a-cai.icp0.io"),
        String::from("https://dduc6-3yaaa-aaaal-ai63a-cai.raw.icp0.io"),
        String::from("https://dduc6-3yaaa-aaaal-ai63a-cai.ic0.app"),
        String::from("https://dduc6-3yaaa-aaaal-ai63a-cai.raw.ic0.app"),
        String::from("https://dduc6-3yaaa-aaaal-ai63a-cai.icp0.icp-api.io"),
        String::from("https://dduc6-3yaaa-aaaal-ai63a-cai.icp-api.io"),
        String::from("https://app.appicdao.com"),
        String::from("https://ib67n-yiaaa-aaaao-qjwca-cai.icp0.io"),
        String::from("https://ib67n-yiaaa-aaaao-qjwca-cai.raw.icp0.io"),
        String::from("https://ib67n-yiaaa-aaaao-qjwca-cai.ic0.app"),
        String::from("https://ib67n-yiaaa-aaaao-qjwca-cai.raw.ic0.app"),
        String::from("https://ib67n-yiaaa-aaaao-qjwca-cai.icp0.icp-api.io"),
        String::from("https://ib67n-yiaaa-aaaao-qjwca-cai.icp-api.io"),
        String::from("https://test.appicdao.com"),
    ];

    return Icrc28TrustedOriginsResponse { trusted_origins };
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
