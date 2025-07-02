// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use std::str::FromStr;

use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;

use crate::{
    numeric::Erc20TokenAmount,
    state::{
        checked_nat_to_erc20_amount, checked_nat_to_u64, checked_nat_to_u8, nat_to_erc20_amount,
        nat_to_u64, nat_to_u8,
        types::{IcpToken, IcpTokenType},
    },
};

#[derive(CandidType, Deserialize, Debug)]
pub struct Config {
    pub value: String,
    pub name: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Media {
    pub link: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct TokenMetadata {
    pub fee: candid::Nat,
    pub configs: Vec<Config>,
    pub decimals: candid::Nat,
    pub name: String,
    pub rank: u32,
    #[serde(rename = "mediaLinks")]
    pub media_links: Vec<Media>,
    #[serde(rename = "totalSupply")]
    pub total_supply: candid::Nat,
    pub introduction: String,
    pub standard: String,
    pub symbol: String,
    #[serde(rename = "canisterId")]
    pub canister_id: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum Result2 {
    #[serde(rename = "ok")]
    Ok(Vec<String>),
    #[serde(rename = "err")]
    Err(String),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TokensListResult {
    #[serde(rename = "ok")]
    Ok(Vec<TokenMetadata>),
    #[serde(rename = "err")]
    Err(String),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum GetTokenLogoResult {
    #[serde(rename = "ok")]
    Ok(String),
    #[serde(rename = "err")]
    Err(String),
}

pub struct Service(pub Principal);
impl Service {
    pub async fn get_list(&self) -> Result<(TokensListResult,)> {
        ic_cdk::call(self.0, "getList", ()).await
    }
    pub async fn get_logo(&self, arg0: String) -> Result<(GetTokenLogoResult,)> {
        ic_cdk::call(self.0, "getLogo", (arg0,)).await
    }
}
