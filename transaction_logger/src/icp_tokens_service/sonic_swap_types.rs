// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;

use crate::state::{nat_to_u64, IcpToken, IcpTokenType};

#[derive(CandidType, Deserialize, Debug)]
pub struct TokenInfoWithType {
    pub id: String,
    pub fee: candid::Nat,
    pub decimals: u8,
    pub name: String,
    #[serde(rename = "totalSupply")]
    pub total_supply: candid::Nat,
    #[serde(rename = "blockStatus")]
    pub block_status: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    pub symbol: String,
}

impl From<TokenInfoWithType> for IcpToken {
    fn from(value: TokenInfoWithType) -> Self {
        let token_type = match value.token_type.as_str() {
            "ICRC1" => IcpTokenType::ICRC1,
            "ICRC2" => IcpTokenType::ICRC2,
            "ICRC3" => IcpTokenType::ICRC3,
            "DIP20" => IcpTokenType::DIP20,
            _ => IcpTokenType::Other(value.token_type),
        };
        let ledger_id = Principal::from_text(value.id).unwrap_or(Principal::anonymous());

        Self {
            ledger_id,
            name: value.name,
            decimals: value.decimals,
            symbol: value.symbol,
            token_type,
            fee: nat_to_u64(&value.fee),
            rank: None,
        }
    }
}

pub struct Service(pub Principal);
impl Service {
    pub async fn get_supported_token_list(&self) -> Result<(Vec<TokenInfoWithType>,)> {
        ic_cdk::call(self.0, "getSupportedTokenList", ()).await
    }
}
