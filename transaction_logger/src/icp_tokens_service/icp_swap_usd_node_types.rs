// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
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

#[allow(non_snake_case)]
#[derive(CandidType, Deserialize)]
pub struct Transaction {
    pub to: String,
    pub action: TransactionType,
    pub token0Id: String,
    pub token1Id: String,
    pub liquidityTotal: candid::Nat,
    pub from: String,
    pub hash: String,
    pub tick: candid::Int,
    pub token1Price: f64,
    pub recipient: String,
    pub token0ChangeAmount: f64,
    pub sender: String,
    pub liquidityChange: candid::Nat,
    pub token1Standard: String,
    pub token0Fee: f64,
    pub token1Fee: f64,
    pub timestamp: candid::Int,
    pub token1ChangeAmount: f64,
    pub token1Decimals: f64,
    pub token0Standard: String,
    pub amountUSD: f64,
    pub amountToken0: f64,
    pub amountToken1: f64,
    pub poolFee: candid::Nat,
    pub token0Symbol: String,
    pub token0Decimals: f64,
    pub token0Price: f64,
    pub token1Symbol: String,
    pub poolId: String,
}

#[derive(CandidType, Deserialize)]
pub enum NatResult {
    #[serde(rename = "ok")]
    Ok(candid::Nat),
    #[serde(rename = "err")]
    Err(String),
}

#[allow(non_snake_case)]
#[derive(CandidType, Deserialize)]
pub struct PublicPoolOverView {
    pub id: candid::Nat,
    pub token0TotalVolume: f64,
    pub volumeUSD1d: f64,
    pub volumeUSD7d: f64,
    pub token0Id: String,
    pub token1Id: String,
    pub token1Volume24H: f64,
    pub totalVolumeUSD: f64,
    pub sqrtPrice: f64,
    pub pool: String,
    pub tick: candid::Int,
    pub liquidity: candid::Nat,
    pub token1Price: f64,
    pub feeTier: candid::Nat,
    pub token1TotalVolume: f64,
    pub volumeUSD: f64,
    pub feesUSD: f64,
    pub token0Volume24H: f64,
    pub token1Standard: String,
    pub txCount: candid::Nat,
    pub token1Decimals: f64,
    pub token0Standard: String,
    pub token0Symbol: String,
    pub token0Decimals: f64,
    pub token0Price: f64,
    pub token1Symbol: String,
}

#[allow(non_snake_case)]
#[derive(CandidType, Deserialize, Debug)]
pub struct PublicTokenOverview {
    pub id: candid::Nat,
    pub volumeUSD1d: f64,
    pub volumeUSD7d: f64,
    pub totalVolumeUSD: f64,
    pub name: String,
    pub volumeUSD: f64,
    pub feesUSD: f64,
    pub priceUSDChange: f64,
    pub address: String,
    pub txCount: candid::Int,
    pub priceUSD: f64,
    pub standard: String,
    pub symbol: String,
}

#[derive(CandidType, Deserialize)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<HeaderField>,
}

#[derive(CandidType, Deserialize)]
pub struct Token {
    pub arbitrary_data: String,
}

#[derive(CandidType, Deserialize)]
pub struct StreamingCallbackHttpResponse {
    pub token: Option<Token>,
    pub body: serde_bytes::ByteBuf,
}

candid::define_function!(pub CallbackStrategyCallback : (Token) -> (
    StreamingCallbackHttpResponse,
  ) query);
#[derive(CandidType, Deserialize)]
pub struct CallbackStrategy {
    pub token: Token,
    pub callback: CallbackStrategyCallback,
}

#[derive(CandidType, Deserialize)]
pub enum StreamingStrategy {
    Callback(CallbackStrategy),
}

#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<HeaderField>,
    pub upgrade: Option<bool>,
    pub streaming_strategy: Option<StreamingStrategy>,
    pub status_code: u16,
}

pub type Address = String;
