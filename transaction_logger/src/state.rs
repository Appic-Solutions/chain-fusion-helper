use crate::address::Address;
use crate::logs::INFO;
use crate::numeric::LedgerMintIndex;
use crate::state::types::*;

use candid::{CandidType, Nat, Principal};
use ic_canister_log::log;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::BTreeMap;
use ic_stable_structures::DefaultMemoryImpl;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
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

use crate::minter_client::appic_minter_types::events::{TransactionReceipt, TransactionStatus};

mod config;
mod storable_impl;
pub mod types;

use config::{
    erc20_twin_ledger_requests_id, evm_to_icp_memory, evm_token_list_id, icp_to_evm_memory,
    icp_token_list_id, minter_memory, supported_appic_tokens_memory_id,
    supported_ckerc20_tokens_memory_id,
};

// State Definition,
// All types of transactions will be sorted in this stable state
pub struct State {
    // List of all minters including (ckEth dfinity and appic minters)
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
    pub fn update_minter_fees(&mut self, minter_key: &MinterKey, icp_to_evm_fee: Erc20TokenAmount) {
        if let Some(minter) = self.minters.get(minter_key) {
            let new_minter = Minter {
                icp_to_evm_fee,
                ..minter
            };
            self.record_minter(new_minter);
        }
    }

    pub fn enable_minter(&mut self, minter_key: &MinterKey) {
        if let Some(minter) = self.minters.get(minter_key) {
            let updated_minter = Minter {
                enabled: true,
                ..minter
            };
            self.record_minter(updated_minter);
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

    pub fn get_active_minters(&self) -> Vec<(MinterKey, Minter)> {
        self.minters
            .iter()
            .filter(|minter| minter.1.enabled == true)
            .collect()
    }

    pub fn if_chain_id_exists(&self, chain_id: ChainId) -> bool {
        self.minters
            .iter()
            .any(|(_key, minter)| minter.chain_id == chain_id)
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
        ledger_mint_index: LedgerMintIndex,
        transfer_fee: Option<Nat>,
    ) {
        if let Some(tx) = self.evm_to_icp_txs.get(&identifier) {
            // Fee calculation
            let transfer_fee = nat_to_erc20_amount(transfer_fee.unwrap_or(Nat::from(0_u8)));
            let actual_received = Some(
                tx.value
                    .checked_sub(transfer_fee)
                    .unwrap_or(Erc20TokenAmount::ZERO),
            );

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
        l1_fee: Option<Nat>,
        withdrawal_fee: Option<Nat>,
    ) {
        let l1_fee = nat_to_erc20_amount(l1_fee.unwrap_or(Nat::from(0_u8)));
        let withdrawal_fee = nat_to_erc20_amount(withdrawal_fee.unwrap_or(Nat::from(0_u8)));
        let max_transaction_fee =
            nat_to_erc20_amount(max_transaction_fee.unwrap_or(Nat::from(0_u8)));

        let destination_address = Address::from_str(&destination)
            .expect("Should not fail converting destination to Address");
        let erc20_address = Address::from_str(&erc20_contract_address)
            .expect("Should not fail converting ERC20 contract address to Address");
        let total_transaction_fee = l1_fee
            .checked_add(withdrawal_fee)
            .unwrap()
            .checked_add(max_transaction_fee)
            .unwrap_or(Erc20TokenAmount::MAX);

        let withdrawal_amount = nat_to_erc20_amount(withdrawal_amount);

        let native_ledger_burn_index = LedgerBurnIndex::from(nat_to_u64(&native_ledger_burn_index));

        let erc20_ledger_burn_index = erc20_ledger_burn_index
            .map(|burn_index| LedgerBurnIndex::from(nat_to_u64(&burn_index)));

        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let new_tx = IcpToEvmTx {
                verified: true,
                max_transaction_fee: Some(total_transaction_fee),
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
                max_transaction_fee: Some(total_transaction_fee),
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
    ) {
        if let Some(tx) = self.icp_to_evm_txs.get(&identifier) {
            let gas_used = nat_to_erc20_amount(receipt.gas_used);
            let effective_gas_price = nat_to_erc20_amount(receipt.effective_gas_price);

            let total_gas_spent = gas_used.checked_mul(effective_gas_price).unwrap();

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

    pub fn record_native_icrc_ledger(
        &mut self,
        ledger: Principal,
        symbol: String,
        transfer_fee: Erc20TokenAmount,
        chain_id: ChainId,
    ) {
        log!(
            INFO,
            "Recieved native ledger for chain_id:{:?} with principal:{:?}",
            chain_id,
            ledger.to_text()
        );

        let evm_token = self
            .get_evm_token_by_identifier(&Erc20Identifier(Address::ZERO, chain_id))
            .expect("Native token should already be available");
        let icp_token = IcpToken {
            ledger_id: ledger,
            name: symbol.clone(),
            decimals: 18,
            symbol: symbol,
            usd_price: "0.01".to_string(),
            logo: evm_token.logo,
            fee: transfer_fee,
            token_type: IcpTokenType::ICRC2,
            rank: Some(1),
        };
        self.record_icp_token(ledger, icp_token);
    }

    pub fn record_deployed_wrapped_icrc_token(
        &mut self,
        icrc_token: Principal,
        wrapped_token: Erc20Identifier,
    ) {
        log!(
            INFO,
            "Recieved deployed wrapped_icrc event {:?}",
            icrc_token.to_text()
        );
        match self.get_icp_token_by_principal(&icrc_token) {
            Some(token) => {
                let evm_token = EvmToken {
                    chain_id: wrapped_token.chain_id(),
                    erc20_contract_address: wrapped_token.erc20_address(),
                    name: token.name.clone(),
                    decimals: token.decimals,
                    symbol: token.symbol.clone(),
                    logo: token.logo.clone(),
                    is_wrapped_icrc: true,
                };
                self.evm_token_list
                    .insert(wrapped_token.clone(), evm_token.clone());

                self.supported_twin_appic_tokens.insert(
                    wrapped_token,
                    BridgePair {
                        icp_token: token,
                        evm_token,
                    },
                );
            }
            None => {}
        };
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
    pub fn get_supported_bridge_pairs(&self) -> Vec<TokenPair> {
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
        == &Address::from_str(NATIVE_ERC20_ADDRESS).expect("Should not fail converting to address")
}

pub fn nat_to_ledger_burn_index(value: &Nat) -> LedgerBurnIndex {
    LedgerBurnIndex::from(nat_to_u64(value))
}

pub fn nat_to_ledger_mint_index(value: &Nat) -> LedgerMintIndex {
    LedgerMintIndex::from(nat_to_u64(value))
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
