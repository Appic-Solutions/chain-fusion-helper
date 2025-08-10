use base64::{engine::general_purpose, Engine as _};
use candid::Principal;
use ic_canister_log::log;
use ic_cdk::{init, post_upgrade, query, update};
use ic_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use std::borrow::Borrow;
use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;
use transaction_logger::add_evm_tokens::add_evm_tokens_to_state;
use transaction_logger::address::Address;
use transaction_logger::appic_dex_types::CandidEvent;
use transaction_logger::endpoints::{
    AddEvmToIcpTx, AddEvmToIcpTxError, AddIcpToEvmTx, AddIcpToEvmTxError,
    CandidAddErc20TwinLedgerSuiteRequest, CandidEvmToken, CandidIcpToken, CandidLedgerSuiteRequest,
    EvmSearchQuery, GetEvmTokenArgs, GetIcpTokenArgs, GetTxParams, Icrc28TrustedOriginsResponse,
    MinterArgs, TokenPair, TopVolumeTokens, Transaction,
};
use transaction_logger::guard::{TaskType, TimerGuard};
use transaction_logger::lifecycle::{self, init as initialize};
use transaction_logger::scrape_dex_events::scrape_dex_events;
use transaction_logger::state::{
    mutate_state, nat_to_erc20_amount, nat_to_ledger_burn_index, read_state,
    types::{
        ChainId, Erc20Identifier, Erc20TwinLedgerSuiteRequest, EvmToIcpStatus, EvmToIcpTx,
        EvmToIcpTxIdentifier, EvmToken, IcpToEvmIdentifier, IcpToEvmStatus, IcpToEvmTx, IcpToken,
    },
};
use transaction_logger::update_bridge_pairs::APPIC_LEDGER_MANAGER_ID;
use transaction_logger::update_icp_tokens::{update_icp_tokens, update_usd_price, validate_tokens};
use transaction_logger::{
    endpoints::LoggerArgs, logs::INFO, remove_unverified_tx::remove_unverified_tx,
    scrape_events::scrape_events, update_bridge_pairs::update_bridge_pairs, REMOVE_UNVERIFIED_TX,
    SCRAPE_EVENTS, UPDATE_BRIDGE_PAIRS,
};
use transaction_logger::{REMOVE_INVALID_ICP_TOKENS, UPDATE_ICP_TOKENS, UPDATE_USD_PRICE};

const ADMIN_ID: &str = "tb3vi-54bcb-4oudm-fmp2s-nntjp-rmhd3-ukvnq-lawfq-vk5vy-mnlc7-pae";
const DATA_PROVIDER_ID: &str = "o74ab-rm2co-uhvn6-6ec2d-3kkvk-bwlcw-356yj-lbma2-m4qew-l4ett-wae";

// Setup timers
fn setup_timers() {
    // Start scraping events.
    ic_cdk_timers::set_timer_interval(SCRAPE_EVENTS, || ic_cdk::spawn(scrape_events()));

    //ic_cdk_timers::set_timer_interval(SCRAPE_EVENTS, || ic_cdk::spawn(scrape_dex_events()));

    // Update usd price of icp tokens
    ic_cdk_timers::set_timer_interval(UPDATE_USD_PRICE, || ic_cdk::spawn(update_usd_price()));

    // Remove unverified transactions
    ic_cdk_timers::set_timer_interval(REMOVE_UNVERIFIED_TX, remove_unverified_tx);

    // Check new supported twin tokens
    ic_cdk_timers::set_timer_interval(UPDATE_BRIDGE_PAIRS, || ic_cdk::spawn(update_bridge_pairs()));

    // Update Icp token list
    ic_cdk_timers::set_timer_interval(UPDATE_ICP_TOKENS, || ic_cdk::spawn(update_icp_tokens()));

    // Remove invalid icp tokens
    ic_cdk_timers::set_timer_interval(REMOVE_INVALID_ICP_TOKENS, || {
        ic_cdk::spawn(validate_tokens())
    });
}

fn is_authorized_caller(caller: Principal) -> bool {
    let appic_ledger_manager_id =
        Principal::from_text(APPIC_LEDGER_MANAGER_ID).expect("Invalid APPIC_LEDGER_MANAGER_ID");
    let admin_id = Principal::from_text(ADMIN_ID).expect("Invalid ADMIN_ID");

    let appic_data_provider_id =
        Principal::from_text(DATA_PROVIDER_ID).expect("Invalid DATA_PROVIDER_ID");

    caller == appic_ledger_manager_id || caller == admin_id || caller == appic_data_provider_id
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

    setup_timers();
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
    // Ensures that scraping events will be blocked and
    // All tokens are added to canister state
    let _guard_scraping_events: TimerGuard =
        TimerGuard::new(TaskType::ScrapeEvents).expect("No guard should exist at this point");

    update_icp_tokens().await;
    update_usd_price().await;
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

    add_evm_tokens_to_state();

    // Set up timers
    setup_timers();
}

// Add new icp to evm transaction
#[update]
fn new_icp_to_evm_tx(tx: AddIcpToEvmTx) -> Result<(), AddIcpToEvmTxError> {
    let tx_identifier = IcpToEvmIdentifier::from(&tx);
    let chain_id = ChainId::from(&tx.chain_id);

    if read_state(|s| s.if_icp_to_evm_tx_exists(&tx_identifier)) {
        return Err(AddIcpToEvmTxError::TxAlreadyExists);
    };

    if !read_state(|s| s.if_chain_id_exists(chain_id)) {
        return Err(AddIcpToEvmTxError::ChainNotSupported);
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
            None => Err(AddIcpToEvmTxError::InvalidTokenPairs),
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
                time: ic_cdk::api::time(),
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

    if read_state(|s| s.if_evm_to_icp_tx_exists(&tx_identifier)) {
        return Err(AddEvmToIcpTxError::TxAlreadyExists);
    };

    if !read_state(|s| s.if_chain_id_exists(chain_id)) {
        return Err(AddEvmToIcpTxError::ChainNotSupported);
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
            None => Err(AddEvmToIcpTxError::InvalidTokenPairs),
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
pub fn get_txs_by_address_principal_combination(
    address: String,
    principal_id: Principal,
) -> Vec<Transaction> {
    let address = Address::from_str(&address).expect("Address should be valid");

    // get transactions by address and principal
    let txs_by_address = read_state(|s| s.get_transaction_for_address(address));
    let txs_by_principal = read_state(|s| s.get_transaction_for_principal(principal_id));

    // Use a HashSet to remove duplicates
    let mut unique_txs_set: HashSet<Transaction> = HashSet::new();
    txs_by_address.into_iter().for_each(|tx| {
        unique_txs_set.insert(tx);
    });
    txs_by_principal.into_iter().for_each(|tx| {
        unique_txs_set.insert(tx);
    });

    // Convert the HashSet back to a Vec and return
    unique_txs_set.into_iter().collect()
}
#[query]
pub fn get_bridge_pairs() -> Vec<TokenPair> {
    read_state(|s| s.get_supported_bridge_pairs())
}

#[query]
pub fn get_transaction(params: GetTxParams) -> Option<Transaction> {
    // Check if chain id is supported
    let chain_id = ChainId::from(&params.chain_id);
    let chain_check_result = read_state(|s| s.if_chain_id_exists(chain_id));

    if !chain_check_result {
        return None;
    }

    read_state(|s| s.get_transaction_by_search_params(params.search_param, chain_id))
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

#[update]
// Can only be called by lsm
pub fn add_icp_token(token: CandidIcpToken) {
    if !is_authorized_caller(ic_cdk::caller()) {
        panic!("Only admins can change icp tokens details")
    }

    let token: IcpToken = token.into();
    mutate_state(|s| s.record_icp_token(token.ledger_id, token))
}

#[update]
// Can only be called by admin
pub fn add_evm_token(token: CandidEvmToken) {
    if !is_authorized_caller(ic_cdk::caller()) {
        panic!("Only admins can change icp tokens details")
    }

    let token: EvmToken = token.into();
    mutate_state(|s| {
        s.record_evm_token(
            Erc20Identifier::new(&token.erc20_contract_address, token.chain_id),
            token,
        )
    })
}

#[update]
// can only be called by
// arguments: (Vec<(cmc_id,volume,price)>)
pub fn update_evm_token_price_volume(data: Vec<(u64, String, String)>) {
    if !is_authorized_caller(ic_cdk::caller()) {
        panic!("Only admins can change icp tokens details")
    }

    mutate_state(|s| s.update_evm_price_volume_by_cmc_id(data))
}

#[query]
pub fn get_top_100_tokens_by_volume_per_chain() -> Vec<TopVolumeTokens> {
    read_state(|s| s.get_top_100_tokens_by_volume_per_chain())
        .into_iter()
        .map(|(chain, tokens)| TopVolumeTokens {
            chain,
            tokens: tokens.into_iter().map(|token| token.into()).collect(),
        })
        .collect()
}

#[query]
// either symbol, name or the contract address of that token
pub fn search_evm_token(query: EvmSearchQuery) -> Vec<CandidEvmToken> {
    read_state(|s| s.search_evm_token(query.chain_id, query.query))
        .into_iter()
        .map(|token| token.into())
        .collect()
}

#[query]
pub fn get_icp_tokens() -> Vec<CandidIcpToken> {
    // Get tokens from state
    let tokens = read_state(|s| s.get_icp_tokens());

    // Return Tokens
    tokens.into_iter().map(CandidIcpToken::from).collect()
}

// Can only be called by lsm
#[update]
pub fn new_twin_ls_request(request: CandidAddErc20TwinLedgerSuiteRequest) {
    if !is_authorized_caller(ic_cdk::caller()) {
        panic!("Only admins can change twin token details")
    }
    let erc20_identifier: Erc20Identifier = request.borrow().into();
    let erc20_twin_ls_request: Erc20TwinLedgerSuiteRequest = request.into();

    mutate_state(|s| {
        s.twin_erc20_requests
            .insert(erc20_identifier, erc20_twin_ls_request)
    });
}

// Can only be called by lsm
#[update]
pub fn update_twin_ls_request(updated_request: CandidAddErc20TwinLedgerSuiteRequest) {
    if !is_authorized_caller(ic_cdk::caller()) {
        panic!("Only admins can change twin token details")
    }
    let erc20_identifier: Erc20Identifier = updated_request.borrow().into();
    let erc20_twin_ls_request: Erc20TwinLedgerSuiteRequest = updated_request.into();

    mutate_state(|s| {
        s.twin_erc20_requests
            .insert(erc20_identifier, erc20_twin_ls_request)
    });
}

// Can only be called by lsm
#[update]
pub async fn request_update_bridge_pairs() {
    if !is_authorized_caller(ic_cdk::caller()) {
        panic!("Only admins request update bride pairs")
    }
    update_bridge_pairs().await;
}

#[query]
pub fn get_erc20_twin_ls_requests_by_creator(creator: Principal) -> Vec<CandidLedgerSuiteRequest> {
    let requests = read_state(|s| s.get_erc20_ls_requests_by_principal(creator));
    requests.into_iter().map(|request| request.into()).collect()
}

// Get minters
#[query]
pub fn get_minters() -> Vec<MinterArgs> {
    read_state(|s| s.get_minters())
        .into_iter()
        .map(|(_key, minter)| minter.to_candid_minter_args())
        .collect()
}

#[query]
pub fn get_dex_actions_for_principal(principal_id: Principal) -> Vec<CandidEvent> {
    read_state(|s| s.get_dex_actions_for_principal(principal_id))
        .into_iter()
        .map(|dex_action| CandidEvent::from((principal_id, dex_action)))
        .collect()
}

#[query(hidden = true)]
fn http_request(request: HttpRequest) -> HttpResponse {
    let path = request.url.trim_start_matches('/');
    let path_parts: Vec<&str> = path.split('/').collect();

    if path_parts.len() == 2 && path_parts[0] == "logo" {
        match Principal::from_text(path_parts[1]) {
            Ok(ledger_id) => match read_state(|s| s.get_icp_token_by_principal(&ledger_id)) {
                Some(token) => {
                    let parts: Vec<&str> = token.logo.splitn(2, ';').collect();
                    let content_type = if !parts.is_empty() && parts[0].starts_with("data:") {
                        parts[0].strip_prefix("data:").unwrap_or("image/png")
                    } else {
                        "image/png" // Fallback
                    };

                    // Extract base64 data from the data URL
                    let base64_start = token.logo.find("base64,").map(|pos| pos + 7).unwrap_or(0);
                    let base64_data = &token.logo[base64_start..];

                    // Decode base64 to binary
                    match general_purpose::STANDARD.decode(base64_data) {
                        Ok(decoded) => HttpResponseBuilder::ok()
                            .header("Content-Type", content_type)
                            .body(decoded)
                            .build(),

                        Err(e) => {
                            HttpResponseBuilder::server_error(format!("Base64 decode error: {e}"))
                                .body(format!("Base64 decode error: {e}"))
                                .build()
                        }
                    }
                }
                None => HttpResponseBuilder::not_found().build(),
            },
            Err(_) => HttpResponseBuilder::bad_request().build(),
        }
    } else {
        HttpResponseBuilder::bad_request().build()
    }
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

    Icrc28TrustedOriginsResponse { trusted_origins }
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
