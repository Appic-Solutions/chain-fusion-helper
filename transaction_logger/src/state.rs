use crate::numeric::LedgerMintIndex;
use candid::{CandidType, Nat, Principal};
use ic_ethereum_types::Address;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{storable::Bound, BTreeMap, Storable};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};

use std::str::FromStr;

use crate::endpoints::{
    AddEvmToIcpTx, AddIcpToEvmTx, CandidErc20TwinLedgerSuiteFee, CandidErc20TwinLedgerSuiteStatus,
    CandidEvmToIcp, CandidEvmToken, CandidIcpToEvm, CandidIcpToken, CandidLedgerSuiteRequest,
    MinterArgs, TokenPair, Transaction, TransactionSearchParam,
};
use crate::numeric::{BlockNumber, Erc20TokenAmount, LedgerBurnIndex};
use crate::scrape_events::NATIVE_ERC20_ADDRESS;

use std::fmt::Debug;

use crate::minter_clinet::appic_minter_types::events::{TransactionReceipt, TransactionStatus};

mod config;

use config::{
    erc20_twin_ledger_requests_id, evm_to_icp_memory, evm_token_list_id, icp_to_evm_memory,
    icp_token_list_id, minter_memory, supported_appic_tokens_memory_id,
    supported_ckerc20_tokens_memory_id,
};

#[derive(
    Clone, Copy, CandidType, PartialEq, PartialOrd, Eq, Ord, Debug, Deserialize, Serialize,
)]
pub enum Operator {
    DfinityCkEthMinter,
    AppicMinter,
}

#[derive(Clone, PartialEq, Ord, PartialOrd, Eq, Debug, Deserialize, Serialize)]
pub struct Minter {
    pub id: Principal,
    pub last_observed_event: u64,
    pub last_scraped_event: u64,
    pub operator: Operator,
    pub evm_to_icp_fee: Erc20TokenAmount,
    pub icp_to_evm_fee: Erc20TokenAmount,
    pub chain_id: ChainId,
}

impl Minter {
    pub fn update_last_observed_event(&mut self, event: u64) {
        self.last_observed_event = event
    }

    pub fn update_last_scraped_event(&mut self, event: u64) {
        self.last_scraped_event = event
    }

    pub fn from_minter_args(args: MinterArgs) -> Self {
        let MinterArgs {
            chain_id,
            minter_id,
            operator,
            last_observed_event,
            last_scraped_event,
            evm_to_icp_fee,
            icp_to_evm_fee,
        } = args;
        Self {
            id: minter_id,
            last_observed_event: nat_to_u64(&last_observed_event),
            last_scraped_event: nat_to_u64(&last_scraped_event),
            operator,
            evm_to_icp_fee: Erc20TokenAmount::try_from(evm_to_icp_fee)
                .expect("Should not fail converting fees"),
            icp_to_evm_fee: Erc20TokenAmount::try_from(icp_to_evm_fee)
                .expect("Should not fail converting fees"),
            chain_id: ChainId::from(&chain_id),
        }
    }
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct MinterKey(pub ChainId, pub Operator);

impl MinterKey {
    pub fn operator(&self) -> Operator {
        self.1
    }

    pub fn chain_id(&self) -> ChainId {
        self.0
    }
}

impl From<&Minter> for MinterKey {
    fn from(value: &Minter) -> Self {
        Self(value.chain_id, value.operator)
    }
}

type TransactionHash = String;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub struct EvmToIcpTxIdentifier(TransactionHash, ChainId);

impl EvmToIcpTxIdentifier {
    /// Creates a new `EvmToIcpTxIdentifier` instance.
    pub fn new(transaction_hash: &TransactionHash, chain_id: ChainId) -> Self {
        Self(transaction_hash.clone(), chain_id)
    }
}

impl From<&AddEvmToIcpTx> for EvmToIcpTxIdentifier {
    fn from(value: &AddEvmToIcpTx) -> Self {
        Self::new(&value.transaction_hash, ChainId::from(&value.chain_id))
    }
}

#[derive(Clone, CandidType, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum EvmToIcpStatus {
    PendingVerification,
    Accepted,
    Minted,
    Invalid(String),
    Quarantined,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct EvmToIcpTx {
    pub from_address: Address,
    pub transaction_hash: TransactionHash,
    pub value: Erc20TokenAmount,
    pub ledger_mint_index: Option<LedgerMintIndex>,
    pub block_number: Option<BlockNumber>,
    pub actual_received: Option<Erc20TokenAmount>,
    pub principal: Principal,
    pub subaccount: Option<[u8; 32]>,
    pub chain_id: ChainId,
    pub total_gas_spent: Option<Erc20TokenAmount>,
    pub erc20_contract_address: Address,
    pub icrc_ledger_id: Option<Principal>,
    pub status: EvmToIcpStatus,
    pub verified: bool,
    pub time: u64,
    pub operator: Operator,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub struct IcpToEvmIdentifier(LedgerBurnIndex, ChainId);

impl IcpToEvmIdentifier {
    /// Creates a new `IcpToEvmIdentifier` instance.
    pub fn new(ledger_burn_index: LedgerBurnIndex, chain_id: ChainId) -> Self {
        Self(ledger_burn_index, chain_id)
    }
}

impl From<&AddIcpToEvmTx> for IcpToEvmIdentifier {
    fn from(value: &AddIcpToEvmTx) -> Self {
        let ledger_burn_index = LedgerBurnIndex::new(nat_to_u64(&value.native_ledger_burn_index));
        let chain_id = ChainId::from(&value.chain_id);
        Self::new(ledger_burn_index, chain_id)
    }
}

#[derive(CandidType, Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum IcpToEvmStatus {
    PendingVerification,
    Accepted,
    Created,
    SignedTransaction,
    FinalizedTransaction,
    ReplacedTransaction,
    Reimbursed,
    QuarantinedReimbursement,
    Successful,
    Failed,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct IcpToEvmTx {
    pub transaction_hash: Option<TransactionHash>,
    pub native_ledger_burn_index: LedgerBurnIndex,
    pub withdrawal_amount: Erc20TokenAmount,
    pub actual_received: Option<Erc20TokenAmount>,
    pub destination: Address,
    pub from: Principal,
    pub chain_id: ChainId,
    pub from_subaccount: Option<[u8; 32]>,
    pub time: u64,
    pub max_transaction_fee: Option<Erc20TokenAmount>,
    pub effective_gas_price: Option<Erc20TokenAmount>,
    pub gas_used: Option<Erc20TokenAmount>,
    pub total_gas_spent: Option<Erc20TokenAmount>,
    pub erc20_ledger_burn_index: Option<LedgerBurnIndex>,
    pub erc20_contract_address: Address,
    pub icrc_ledger_id: Option<Principal>,
    pub verified: bool,
    pub status: IcpToEvmStatus,
    pub operator: Operator,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Erc20Identifier(pub Address, pub ChainId);

impl Erc20Identifier {
    pub fn new(contract: &Address, chain_id: ChainId) -> Self {
        Self(*contract, chain_id)
    }

    pub fn erc20_address(&self) -> Address {
        self.0
    }
    pub fn chain_id(&self) -> ChainId {
        self.1
    }
}

impl From<&EvmToken> for Erc20Identifier {
    fn from(value: &EvmToken) -> Self {
        Self::new(&value.erc20_contract_address, value.chain_id)
    }
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct EvmToken {
    pub chain_id: ChainId,
    pub erc20_contract_address: Address,
    pub name: String,
    pub decimals: u8,
    pub symbol: String,
    pub logo: String,
}

#[derive(CandidType, Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum IcpTokenType {
    ICRC1,
    ICRC2,
    ICRC3,
    DIP20,
    Other(String),
}

#[derive(Clone, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub struct IcpToken {
    pub ledger_id: Principal,
    pub name: String,
    pub decimals: u8,
    pub symbol: String,
    pub usd_price: String,
    pub logo: String,
    pub fee: Erc20TokenAmount,
    pub token_type: IcpTokenType,
    pub rank: Option<u32>,
}

// Custom implementation of Eq and Hash for IcpToken based only on ledger_id
impl PartialEq for IcpToken {
    fn eq(&self, other: &Self) -> bool {
        self.ledger_id == other.ledger_id
    }
}

impl Hash for IcpToken {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ledger_id.hash(state);
    }
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BridgePair {
    pub icp_token: IcpToken,
    pub evm_token: EvmToken,
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub enum Erc20TwinLedgerSuiteStatus {
    PendingApproval,
    Created,
    Installed,
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub enum Erc20TwinLedgerSuiteFee {
    Icp(u128),
    Appic(u128),
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Erc20TwinLedgerSuiteRequest {
    pub creator: Principal,
    pub evm_token: Option<EvmToken>,
    pub erc20_contract_address: Address,
    pub chain_id: ChainId,
    pub ledger_id: Option<Principal>,
    pub icp_token_name: String,
    pub icp_token_symbol: String,
    pub icp_token: Option<IcpToken>,
    pub status: Erc20TwinLedgerSuiteStatus,
    pub created_at: u64,
    pub fee_charged: Erc20TwinLedgerSuiteFee,
}

impl From<Erc20TwinLedgerSuiteRequest> for CandidLedgerSuiteRequest {
    fn from(value: Erc20TwinLedgerSuiteRequest) -> Self {
        let status: CandidErc20TwinLedgerSuiteStatus = value.status.into();
        let fee_charged: CandidErc20TwinLedgerSuiteFee = value.fee_charged.into();

        Self {
            creator: value.creator,
            evm_token: value.evm_token.map(|token| token.into()),
            icp_token: value.icp_token.map(|token| token.into()),
            erc20_contract: value.erc20_contract_address.to_string(),
            chain_id: value.chain_id.into(),
            status,
            created_at: value.created_at,
            fee_charged,
        }
    }
}

// State Definition,
// All types of transactions will be sotred in this stable state
pub struct State {
    // List of all minters including (cketh dfinity and appic minters)
    pub minters: BTreeMap<MinterKey, Minter, StableMemory>,

    // List of all evm_to_icp transactions
    pub evm_to_icp_txs: BTreeMap<EvmToIcpTxIdentifier, EvmToIcpTx, StableMemory>,

    // list of all icp_to_evm transactions
    pub icp_to_evm_txs: BTreeMap<IcpToEvmIdentifier, IcpToEvmTx, StableMemory>,

    pub supported_ckerc20_tokens: BTreeMap<Erc20Identifier, BridgePair, StableMemory>,
    pub supported_twin_appic_tokens: BTreeMap<Erc20Identifier, BridgePair, StableMemory>,

    pub evm_token_list: BTreeMap<Erc20Identifier, EvmToken, StableMemory>,
    pub icp_token_list: BTreeMap<Principal, IcpToken, StableMemory>,

    // List of new erc20 -> icERC20 requests
    pub twin_erc20_requests: BTreeMap<Erc20Identifier, Erc20TwinLedgerSuiteRequest, StableMemory>,
}

impl State {
    pub fn update_minter_fees(
        &mut self,
        minter_key: &MinterKey,
        evm_to_icp_fee: Erc20TokenAmount,
        icp_to_evm_fee: Erc20TokenAmount,
    ) {
        if let Some(minter) = self.minters.get(minter_key) {
            let new_minter = Minter {
                evm_to_icp_fee,
                icp_to_evm_fee,
                ..minter
            };
            self.record_minter(new_minter);
        }
    }

    pub fn update_last_observed_event(&mut self, minter_key: &MinterKey, last_observed_event: u64) {
        if let Some(minter) = self.minters.get(minter_key) {
            let new_minter = Minter {
                last_observed_event,
                ..minter
            };
            self.record_minter(new_minter);
        }
    }

    pub fn update_last_scraped_event(&mut self, minter_key: &MinterKey, last_scraped_event: u64) {
        if let Some(minter) = self.minters.get(minter_key) {
            let new_minter = Minter {
                last_scraped_event,
                ..minter
            };
            self.record_minter(new_minter);
        }
    }

    pub fn get_minters(&self) -> Vec<(MinterKey, Minter)> {
        self.minters.iter().collect()
    }

    pub fn if_chain_id_exists(&self, chain_id: ChainId) -> bool {
        for (_minter_key, minter) in self.get_minters() {
            if minter.chain_id == chain_id {
                return true;
            }
        }
        false
    }

    pub fn record_minter(&mut self, minter: Minter) {
        self.minters.insert(MinterKey::from(&minter), minter);
    }

    pub fn get_icrc_twin_for_erc20(
        &self,
        erc20_identifier: &Erc20Identifier,
        operator: &Operator,
    ) -> Option<Principal> {
        match operator {
            Operator::AppicMinter => self
                .supported_twin_appic_tokens
                .get(erc20_identifier)
                .map(|bridge_pair| bridge_pair.icp_token.ledger_id),
            Operator::DfinityCkEthMinter => self
                .supported_ckerc20_tokens
                .get(erc20_identifier)
                .map(|bridge_pair| bridge_pair.icp_token.ledger_id),
        }
    }

    pub fn if_evm_to_icp_tx_exists(&self, identifier: &EvmToIcpTxIdentifier) -> bool {
        self.evm_to_icp_txs.get(identifier).is_some()
    }

    pub fn if_icp_to_evm_tx_exists(&self, identifier: &IcpToEvmIdentifier) -> bool {
        self.icp_to_evm_txs.get(identifier).is_some()
    }

    pub fn record_new_evm_to_icp(&mut self, identifier: EvmToIcpTxIdentifier, tx: EvmToIcpTx) {
        self.evm_to_icp_txs.insert(identifier, tx);
    }

    pub fn record_accepted_evm_to_icp(
        &mut self,
        identifier: EvmToIcpTxIdentifier,
        transaction_hash: TransactionHash,
        block_number: Nat,
        from_address: String,
        value: Nat,
        principal: Principal,
        erc20_contract_address: String,
        subaccount: Option<[u8; 32]>,
        chain_id: ChainId,
        operator: Operator,
        timestamp: u64,
    ) {
        // Parse addresses once
        let parsed_from_address = Address::from_str(&from_address)
            .expect("Should not fail converting from_address to Address");
        let parsed_erc20_address = Address::from_str(&erc20_contract_address)
            .expect("Should not fail converting erc20_contract_address to Address");

        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            // Update only the necessary fields in the existing transaction
            let new_tx = EvmToIcpTx {
                verified: true,
                block_number: Some(nat_to_block_number(block_number)),
                from_address: parsed_from_address,
                value: nat_to_erc20_amount(value),
                principal,
                erc20_contract_address: parsed_erc20_address,
                subaccount,
                status: EvmToIcpStatus::Accepted,
                ..tx
            };
            self.record_new_evm_to_icp(identifier, new_tx);
        } else {
            // Create a new transaction only if one doses not already exist
            let new_tx = EvmToIcpTx {
                from_address: parsed_from_address,
                transaction_hash,
                value: nat_to_erc20_amount(value),
                block_number: Some(nat_to_block_number(block_number)),
                actual_received: None,
                principal,
                subaccount,
                chain_id,
                total_gas_spent: None,
                erc20_contract_address: parsed_erc20_address,
                icrc_ledger_id: self.get_icrc_twin_for_erc20(
                    &Erc20Identifier(parsed_erc20_address, chain_id),
                    &operator,
                ),
                status: EvmToIcpStatus::Accepted,
                verified: true,
                time: timestamp,
                operator,
                ledger_mint_index: None,
            };

            self.record_new_evm_to_icp(identifier, new_tx);
        }
    }

    pub fn record_minted_evm_to_icp(
        &mut self,
        identifier: EvmToIcpTxIdentifier,
        evm_to_icp_fee: Erc20TokenAmount,
        ledger_mint_index: LedgerMintIndex,
    ) {
        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            // Fee calculation
            let actual_received = if is_native_token(&tx.erc20_contract_address) {
                Some(tx.value.checked_sub(evm_to_icp_fee).unwrap_or(tx.value))
            } else {
                Some(tx.value)
            };

            // Transaction update
            let new_tx = EvmToIcpTx {
                actual_received,
                ledger_mint_index: Some(ledger_mint_index),
                status: EvmToIcpStatus::Minted,
                ..tx
            };
            self.record_new_evm_to_icp(identifier, new_tx);
        }
    }

    pub fn record_invalid_evm_to_icp(&mut self, identifier: EvmToIcpTxIdentifier, reason: String) {
        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            let new_tx = EvmToIcpTx {
                status: EvmToIcpStatus::Invalid(reason),
                ..tx
            };
            self.record_new_evm_to_icp(identifier, new_tx);
        }
    }

    pub fn record_quarantined_evm_to_icp(&mut self, identifier: EvmToIcpTxIdentifier) {
        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            let new_tx = EvmToIcpTx {
                status: EvmToIcpStatus::Quarantined,
                ..tx
            };
            self.record_new_evm_to_icp(identifier, new_tx);
        }
    }

    pub fn record_new_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier, tx: IcpToEvmTx) {
        self.icp_to_evm_txs.insert(identifier, tx);
    }

    pub fn record_accepted_icp_to_evm(
        &mut self,
        identifier: IcpToEvmIdentifier,
        max_transaction_fee: Option<Nat>,
        withdrawal_amount: Nat,
        erc20_contract_address: String,
        destination: String,
        native_ledger_burn_index: Nat,
        erc20_ledger_burn_index: Option<Nat>,
        from: Principal,
        from_subaccount: Option<[u8; 32]>,
        created_at: Option<u64>,
        operator: Operator,
        chain_id: ChainId,
        timestamp: u64,
    ) {
        let destination_address = Address::from_str(&destination)
            .expect("Should not fail converting destination to Address");
        let erc20_address = Address::from_str(&erc20_contract_address)
            .expect("Should not fail converting ERC20 contract address to Address");
        let max_transaction_fee = max_transaction_fee.map(|max_fee| nat_to_erc20_amount(max_fee));

        let withdrawal_amount = nat_to_erc20_amount(withdrawal_amount);

        let native_ledger_burn_index = LedgerBurnIndex::new(nat_to_u64(&native_ledger_burn_index));

        let erc20_ledger_burn_index =
            erc20_ledger_burn_index.map(|burn_index| LedgerBurnIndex::new(nat_to_u64(&burn_index)));

        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                verified: true,
                max_transaction_fee,
                withdrawal_amount,
                erc20_contract_address: erc20_address,
                destination: destination_address,
                native_ledger_burn_index,
                erc20_ledger_burn_index,
                from,
                from_subaccount,
                status: IcpToEvmStatus::Accepted,
                ..tx
            };

            self.record_new_icp_to_evm(identifier, new_tx);
        } else {
            let icrc_ledger_id =
                self.get_icrc_twin_for_erc20(&Erc20Identifier(erc20_address, chain_id), &operator);

            let new_tx = IcpToEvmTx {
                native_ledger_burn_index,
                withdrawal_amount,
                actual_received: None,
                destination: destination_address,
                from,
                from_subaccount,
                time: created_at.unwrap_or(timestamp),
                max_transaction_fee,
                erc20_ledger_burn_index,
                icrc_ledger_id,
                chain_id,
                erc20_contract_address: erc20_address,
                verified: true,
                status: IcpToEvmStatus::Accepted,
                operator,
                effective_gas_price: None,
                gas_used: None,
                transaction_hash: None,
                total_gas_spent: None,
            };

            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_created_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::Created,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_signed_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::SignedTransaction,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_replaced_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::ReplacedTransaction,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_finalized_icp_to_evm(
        &mut self,
        identifier: IcpToEvmIdentifier,
        receipt: TransactionReceipt,
        icp_to_evm_fee: Erc20TokenAmount,
    ) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let gas_used = nat_to_erc20_amount(receipt.gas_used);
            let effective_gas_price = nat_to_erc20_amount(receipt.effective_gas_price);

            let total_gas_spent = gas_used
                .checked_mul(effective_gas_price)
                .unwrap()
                .checked_add(icp_to_evm_fee)
                .unwrap();

            let actual_received = if is_native_token(&tx.erc20_contract_address) {
                tx.withdrawal_amount.checked_sub(total_gas_spent)
            } else {
                Some(tx.withdrawal_amount)
            };

            let status = match receipt.status {
                TransactionStatus::Success => IcpToEvmStatus::Successful,
                TransactionStatus::Failure => IcpToEvmStatus::Failed,
            };
            let new_tx = IcpToEvmTx {
                actual_received,
                transaction_hash: Some(receipt.transaction_hash),
                gas_used: Some(gas_used),
                effective_gas_price: Some(effective_gas_price),
                total_gas_spent: Some(total_gas_spent),
                status,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_reimbursed_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::Reimbursed,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn record_quarantined_reimbursed_icp_to_evm(&mut self, identifier: IcpToEvmIdentifier) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                status: IcpToEvmStatus::QuarantinedReimbursement,
                ..tx
            };
            self.record_new_icp_to_evm(identifier, new_tx);
        }
    }

    pub fn all_unverified_icp_to_evm(&self) -> Vec<(IcpToEvmIdentifier, u64)> {
        self.icp_to_evm_txs
            .iter()
            .filter(|(_, tx)| !tx.verified) // Filter out verified transactions
            .map(|(identifier, tx)| (identifier, tx.time)) // Map to the desired tuple
            .collect()
    }

    pub fn remove_unverified_icp_to_evm(&mut self, identifier: &IcpToEvmIdentifier) {
        self.icp_to_evm_txs.remove(identifier);
    }

    pub fn all_unverified_evm_to_icp(&self) -> Vec<(EvmToIcpTxIdentifier, u64)> {
        self.evm_to_icp_txs
            .iter()
            .filter(|(_, tx)| !tx.verified) // Filter out verified transactions
            .map(|(identifier, tx)| (identifier, tx.time)) // Map to the desired tuple
            .collect()
    }

    pub fn remove_unverified_evm_to_icp(&mut self, identifier: &EvmToIcpTxIdentifier) {
        self.evm_to_icp_txs.remove(identifier);
    }

    // Gets all the transaction history for an evm address
    pub fn get_transaction_for_address(&self, address: Address) -> Vec<Transaction> {
        let result: Vec<Transaction> = self
            .evm_to_icp_txs
            .iter()
            .filter(|(_id, tx)| tx.from_address == address)
            .map(|(_id, tx)| Transaction::from(CandidEvmToIcp::from(tx)))
            .chain(
                self.icp_to_evm_txs
                    .iter()
                    .filter(|(_id, tx)| tx.destination == address)
                    .map(|(_id, tx)| Transaction::from(CandidIcpToEvm::from(tx))),
            )
            .collect();

        result
    }

    // Gets all the transaction history for a principal
    pub fn get_transaction_for_principal(&self, principal_id: Principal) -> Vec<Transaction> {
        let result: Vec<Transaction> = self
            .evm_to_icp_txs
            .iter()
            .filter(|(_id, tx)| tx.principal == principal_id)
            .map(|(_id, tx)| Transaction::from(CandidEvmToIcp::from(tx)))
            .chain(
                self.icp_to_evm_txs
                    .iter()
                    .filter(|(_id, tx)| tx.from == principal_id)
                    .map(|(_id, tx)| Transaction::from(CandidIcpToEvm::from(tx))),
            )
            .collect();

        result
    }

    // Gets supported twin token pairs for both Appic and Dfinity NNS Twin tokens
    pub fn get_suported_bridge_pairs(&self) -> Vec<TokenPair> {
        self.supported_ckerc20_tokens
            .values()
            .filter_map(|bridge_pair| {
                // Update usd price
                let icp_token_with_new_usd_price = IcpToken {
                    usd_price: self
                        .get_icp_token_price(&bridge_pair.icp_token.ledger_id)
                        .unwrap_or("0".to_string()),
                    ..bridge_pair.icp_token
                };
                Some(TokenPair {
                    evm_token: CandidEvmToken::from(bridge_pair.evm_token),
                    icp_token: CandidIcpToken::from(icp_token_with_new_usd_price),
                    operator: Operator::DfinityCkEthMinter,
                })
            })
            .chain(
                self.supported_twin_appic_tokens
                    .values()
                    .filter_map(|bridge_pair| {
                        // Update usd price
                        let icp_token_with_new_usd_price = IcpToken {
                            usd_price: self
                                .get_icp_token_price(&bridge_pair.icp_token.ledger_id)
                                .unwrap_or("0".to_string()),
                            ..bridge_pair.icp_token
                        };
                        Some(TokenPair {
                            evm_token: CandidEvmToken::from(bridge_pair.evm_token),
                            icp_token: CandidIcpToken::from(icp_token_with_new_usd_price),
                            operator: Operator::AppicMinter,
                        })
                    }),
            )
            .collect()
    }

    // Searches for a transaction by hash in both evm_to_icp and icp_to_evm
    fn get_transaction_by_hash(&self, tx_hash: &String, chain_id: ChainId) -> Option<Transaction> {
        let evm_to_icp_id = EvmToIcpTxIdentifier::new(tx_hash, chain_id);

        self.evm_to_icp_txs
            .get(&evm_to_icp_id)
            .map(|tx| Transaction::from(CandidEvmToIcp::from(tx)))
            .or_else(|| {
                self.icp_to_evm_txs
                    .values()
                    .find(|tx| {
                        tx.chain_id == chain_id && tx.transaction_hash.as_ref() == Some(tx_hash)
                    })
                    .map(|tx| Transaction::from(CandidIcpToEvm::from(tx)))
            })
    }

    // Searches for a transaction by burn index id in icp_to_evm_tx
    fn get_transaction_by_burn_index(
        &self,
        ledger_burn_index: LedgerBurnIndex,
        chain_id: ChainId,
    ) -> Option<Transaction> {
        let icp_to_evm_id = IcpToEvmIdentifier(ledger_burn_index, chain_id);

        self.icp_to_evm_txs
            .get(&icp_to_evm_id)
            .map(|tx| Transaction::from(CandidIcpToEvm::from(tx)))
    }

    // Searches for a transaction by mint id in evm_to_icp_txs
    fn get_transaction_by_mint_id(
        &self,
        ledger_mint_index: LedgerMintIndex,
        chain_id: ChainId,
    ) -> Option<Transaction> {
        self.evm_to_icp_txs
            .values()
            .find(|tx| tx.chain_id == chain_id && tx.ledger_mint_index == Some(ledger_mint_index))
            .map(|tx| Transaction::EvmToIcp(CandidEvmToIcp::from(tx)))
    }

    // Gets a single transaction by search param
    // Returns none if no transaction is available
    pub fn get_transaction_by_search_params(
        &self,
        search_param: TransactionSearchParam,
        chain_id: ChainId,
    ) -> Option<Transaction> {
        let search_result = match search_param {
            TransactionSearchParam::TxHash(tx_hash) => {
                self.get_transaction_by_hash(&tx_hash, chain_id)
            }

            TransactionSearchParam::TxWithdrawalId(withdrawal_id) => self
                .get_transaction_by_burn_index(nat_to_ledger_burn_index(&withdrawal_id), chain_id),

            TransactionSearchParam::TxMintId(mint_id) => {
                self.get_transaction_by_mint_id(nat_to_ledger_mint_index(&mint_id), chain_id)
            }
        };

        search_result
    }

    // Records a single evm token
    pub fn record_evm_token(&mut self, identifier: Erc20Identifier, token: EvmToken) {
        self.evm_token_list.insert(identifier, token);
    }

    // Records all evm_tokens in bulk
    pub fn record_evm_tokens_bulk(&mut self, tokens: Vec<EvmToken>) {
        tokens.into_iter().for_each(|token| {
            self.evm_token_list
                .insert(Erc20Identifier::from(&token), token);
        });
    }

    // Records a single icp token
    pub fn record_icp_token(&mut self, ledger_id: Principal, token: IcpToken) {
        self.icp_token_list.insert(ledger_id, token);
    }

    // Records all icp_tokens in bulk
    pub fn record_icp_tokens_bulk(&mut self, tokens: Vec<IcpToken>) {
        tokens.into_iter().for_each(|token| {
            self.icp_token_list.insert(token.ledger_id, token);
        });
    }

    pub fn get_evm_token_by_identifier(&self, identifier: &Erc20Identifier) -> Option<EvmToken> {
        self.evm_token_list.get(identifier)
    }

    pub fn get_icp_token_by_principal(&self, ledger_id: &Principal) -> Option<IcpToken> {
        self.icp_token_list.get(ledger_id)
    }

    pub fn get_icp_tokens(&self) -> Vec<IcpToken> {
        self.icp_token_list.values().collect()
    }

    pub fn get_icp_token_price(&self, ledger_id: &Principal) -> Option<String> {
        self.icp_token_list
            .get(ledger_id)
            .map(|token| token.usd_price)
    }

    pub fn remove_icp_token(&mut self, ledger_id: &Principal) {
        self.icp_token_list.remove(ledger_id);
    }

    pub fn update_icp_token_usd_price(&mut self, ledger_id: Principal, new_usd_price: String) {
        if let Some(token) = self.icp_token_list.get(&ledger_id) {
            self.icp_token_list.insert(
                ledger_id,
                IcpToken {
                    usd_price: new_usd_price,
                    ..token
                },
            );
        };
    }

    pub fn get_erc20_ls_requests_by_principal(
        &self,
        principal: Principal,
    ) -> Vec<Erc20TwinLedgerSuiteRequest> {
        self.twin_erc20_requests
            .iter()
            .filter_map(|(_identifier, request)| {
                if request.creator == principal {
                    return Some(request);
                } else {
                    return None;
                }
            })
            .collect()
    }
}

pub fn is_native_token(address: &Address) -> bool {
    address
        == &Address::from_str(NATIVE_ERC20_ADDRESS).expect("Should not fail converintg to address")
}

impl From<&Nat> for ChainId {
    fn from(value: &Nat) -> Self {
        Self(value.0.to_u64().unwrap())
    }
}

impl From<ChainId> for Nat {
    fn from(value: ChainId) -> Self {
        Nat::from(value.0)
    }
}

pub fn nat_to_ledger_burn_index(value: &Nat) -> LedgerBurnIndex {
    LedgerBurnIndex::new(nat_to_u64(value))
}

pub fn nat_to_ledger_mint_index(value: &Nat) -> LedgerMintIndex {
    LedgerMintIndex::new(nat_to_u64(value))
}

pub fn nat_to_block_number(value: Nat) -> BlockNumber {
    BlockNumber::try_from(value).expect("Failed to convert nat into Erc20TokenAmount")
}

pub fn nat_to_erc20_amount(value: Nat) -> Erc20TokenAmount {
    Erc20TokenAmount::try_from(value).expect("Failed to convert nat into Erc20TokenAmount")
}

pub fn checked_nat_to_erc20_amount(value: Nat) -> Option<Erc20TokenAmount> {
    Erc20TokenAmount::try_from(value).ok()
}

pub fn nat_to_u64(value: &Nat) -> u64 {
    value.0.to_u64().unwrap()
}

pub fn checked_nat_to_u64(value: &Nat) -> Option<u64> {
    value.0.to_u64()
}

pub fn nat_to_u128(value: &Nat) -> u128 {
    value.0.to_u128().unwrap()
}

pub fn nat_to_u8(value: &Nat) -> u8 {
    value.0.to_u8().unwrap()
}

pub fn checked_nat_to_u8(value: &Nat) -> Option<u8> {
    value.0.to_u8()
}
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ChainId(pub u64);

impl AsRef<u64> for ChainId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| {
        f(cell
            .borrow()
            .as_ref()
            .expect("BUG: state is not initialized"))
    })
}

// / Mutates (part of) the current state using `f`.
// /
// / Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|cell| {
        f(cell
            .borrow_mut()
            .as_mut()
            .expect("BUG: state is not initialized"))
    })
}

// State configuration
pub type StableMemory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    pub static STATE: RefCell<Option<State>> = RefCell::new(
        Some(State {
                minters: BTreeMap::init(minter_memory()),
                evm_to_icp_txs: BTreeMap::init(evm_to_icp_memory()),
                icp_to_evm_txs: BTreeMap::init(icp_to_evm_memory()),
                supported_ckerc20_tokens: BTreeMap::init(supported_ckerc20_tokens_memory_id()),
                supported_twin_appic_tokens:BTreeMap::init(supported_appic_tokens_memory_id()),
                evm_token_list:BTreeMap::init(evm_token_list_id()),
                icp_token_list:BTreeMap::init(icp_token_list_id()),
                twin_erc20_requests: BTreeMap::init(erc20_twin_ledger_requests_id())

            })
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn compare_bincode_and_ciborium() {
        let tx_identifier: EvmToIcpTxIdentifier = EvmToIcpTxIdentifier(
            "0x8218f324b45a8cd36f38586b062e3884588d926035f08e1dcd3605160b3ebd42".to_string(),
            ChainId(56),
        );

        // Bincode Serialization and Deserialization
        let start = Instant::now();
        let bincode_bytes = bincode::serialize(&tx_identifier).unwrap();
        let bincode_serialization_time = start.elapsed();
        let bincode_size = bincode_bytes.len();

        let start = Instant::now();
        let bincode_deserialized: EvmToIcpTxIdentifier =
            bincode::deserialize(&bincode_bytes).unwrap();
        let bincode_deserialization_time = start.elapsed();

        assert_eq!(bincode_deserialized, tx_identifier);

        // Ciborium Serialization and Deserialization
        let start = Instant::now();
        let mut ciborium_buf = Vec::new();
        ciborium::ser::into_writer(&tx_identifier, &mut ciborium_buf)
            .expect("Failed to serialize with Ciborium");
        let ciborium_serialization_time = start.elapsed();
        let ciborium_size = ciborium_buf.len();

        let start = Instant::now();
        let ciborium_deserialized: EvmToIcpTxIdentifier =
            ciborium::de::from_reader(ciborium_buf.as_slice())
                .expect("Failed to deserialize with Ciborium");
        let ciborium_deserialization_time = start.elapsed();

        assert_eq!(ciborium_deserialized, tx_identifier);

        // Print results
        println!(
            "Bincode - Serialization: {:?}, Deserialization: {:?}, Size: {} bytes",
            bincode_serialization_time, bincode_deserialization_time, bincode_size
        );
        println!(
            "Ciborium - Serialization: {:?}, Deserialization: {:?}, Size: {} bytes",
            ciborium_serialization_time, ciborium_deserialization_time, ciborium_size
        );
    }
}
