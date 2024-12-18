// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;
use serde::Serialize;
#[derive(CandidType, Deserialize, Serialize)]
pub enum TransactionType {
    #[serde(rename = "decreaseLiquidity")]
    DecreaseLiquidity,
    #[serde(rename = "claim")]
    Claim,
    #[serde(rename = "swap")]
    Swap,
    #[serde(rename = "addLiquidity")]
    AddLiquidity,
    #[serde(rename = "increaseLiquidity")]
    IncreaseLiquidity,
}

#[derive(CandidType, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Transaction {
    pub to: String,
    pub action: TransactionType,
    pub token0_id: String,
    pub token1_id: String,
    pub liquidity_total: candid::Nat,
    pub from: String,
    pub hash: String,
    pub tick: candid::Int,
    pub token1_price: f64,
    pub recipient: String,
    pub token0_change_amount: f64,
    pub sender: String,
    pub liquidity_change: candid::Nat,
    pub token1_standard: String,
    pub token0_fee: f64,
    pub token1_fee: f64,
    pub timestamp: candid::Int,
    pub token1_change_amount: f64,
    pub token1_decimals: f64,
    pub token0_standard: String,
    pub amount_usd: f64,
    pub amount_token0: f64,
    pub amount_token1: f64,
    pub pool_fee: candid::Nat,
    pub token0_symbol: String,
    pub token0_decimals: f64,
    pub token0_price: f64,
    pub token1_symbol: String,
    pub pool_id: String,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PublicPoolOverView {
    pub id: candid::Nat,
    pub token0_total_volume: f64,
    pub volume_usd1d: f64,
    pub volume_usd7d: f64,
    pub token0_id: String,
    pub token1_id: String,
    pub token1_volume24h: f64,
    pub total_volume_usd: f64,
    pub sqrt_price: f64,
    pub pool: String,
    pub tick: candid::Int,
    pub liquidity: candid::Nat,
    pub token1_price: f64,
    pub fee_tier: candid::Nat,
    pub token1_total_volume: f64,
    pub volume_usd: f64,
    pub fees_usd: f64,
    pub token0_volume24h: f64,
    pub token1_standard: String,
    pub tx_count: candid::Nat,
    pub token1_decimals: f64,
    pub token0_standard: String,
    pub token0_symbol: String,
    pub token0_decimals: f64,
    pub token0_price: f64,
    pub token1_symbol: String,
}

#[derive(CandidType, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PublicTokenOverview {
    pub id: candid::Nat,
    pub volume_usd1d: f64,
    pub volume_usd7d: f64,
    pub total_volume_usd: f64,
    pub name: String,
    pub volume_usd: f64,
    pub fees_usd: f64,
    pub price_usd_change: f64,
    pub address: String,
    pub tx_count: candid::Int,
    pub price_usd: f64,
    pub standard: String,
    pub symbol: String,
}
