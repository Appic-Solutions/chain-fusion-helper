use super::*;
use minicbor::{Decode, Encode};

// Operator enum (already has Encode, Decode)
#[derive(
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Debug,
    Hash,
    Encode,
    Decode,
    CandidType,
    Serialize,
    Deserialize,
)]
pub enum Operator {
    #[n(0)]
    DfinityCkEthMinter,
    #[n(1)]
    AppicMinter,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Encode, Decode)]
pub struct Minter {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub id: Principal,
    #[n(1)]
    pub last_observed_event: u64,
    #[n(2)]
    pub last_scraped_event: u64,
    #[n(3)]
    pub operator: Operator,
    #[n(4)]
    pub icp_to_evm_fee: Erc20TokenAmount,
    #[n(5)]
    pub chain_id: ChainId,
    #[n(6)]
    pub enabled: bool,
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
        } = args;
        Self {
            id: minter_id,
            last_observed_event: nat_to_u64(&last_observed_event),
            last_scraped_event: nat_to_u64(&last_scraped_event),
            operator,
            icp_to_evm_fee: Erc20TokenAmount::ZERO,
            chain_id: ChainId::from(&chain_id),
            enabled: false,
        }
    }

    pub fn to_candid_minter_args(self) -> MinterArgs {
        MinterArgs {
            chain_id: self.chain_id.into(),
            minter_id: self.id,
            operator: self.operator,
            last_observed_event: self.last_observed_event.into(),
            last_scraped_event: self.last_scraped_event.into(),
        }
    }
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Encode, Decode)]
pub struct MinterKey(#[n(0)] pub ChainId, #[n(1)] pub Operator);

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

pub type TransactionHash = String;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Encode, Decode)]
pub struct EvmToIcpTxIdentifier(#[n(0)] pub TransactionHash, #[n(1)] pub ChainId);

impl EvmToIcpTxIdentifier {
    pub fn new(transaction_hash: &TransactionHash, chain_id: ChainId) -> Self {
        Self(transaction_hash.clone(), chain_id)
    }
}

impl From<&AddEvmToIcpTx> for EvmToIcpTxIdentifier {
    fn from(value: &AddEvmToIcpTx) -> Self {
        Self::new(&value.transaction_hash, ChainId::from(&value.chain_id))
    }
}

#[derive(
    Clone,
    PartialEq,
    Ord,
    Eq,
    PartialOrd,
    Debug,
    Hash,
    Encode,
    Decode,
    CandidType,
    Serialize,
    Deserialize,
)]
pub enum EvmToIcpStatus {
    #[n(0)]
    PendingVerification,
    #[n(1)]
    Accepted,
    #[n(2)]
    Minted,
    #[n(3)]
    Invalid(#[n(0)] String),
    #[n(4)]
    Quarantined,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Encode, Decode)]
pub struct EvmToIcpTx {
    #[n(0)]
    pub from_address: Address,
    #[n(1)]
    pub transaction_hash: TransactionHash,
    #[n(2)]
    pub value: Erc20TokenAmount,
    #[n(3)]
    pub ledger_mint_index: Option<LedgerMintIndex>,
    #[n(4)]
    pub block_number: Option<BlockNumber>,
    #[n(5)]
    pub actual_received: Option<Erc20TokenAmount>,
    #[cbor(n(6), with = "crate::cbor::principal")]
    pub principal: Principal,
    #[n(7)]
    pub subaccount: Option<[u8; 32]>,
    #[n(8)]
    pub chain_id: ChainId,
    #[n(9)]
    pub total_gas_spent: Option<Erc20TokenAmount>,
    #[n(10)]
    pub erc20_contract_address: Address,
    #[cbor(n(11), with = "crate::cbor::principal::option")]
    pub icrc_ledger_id: Option<Principal>,
    #[n(12)]
    pub status: EvmToIcpStatus,
    #[n(13)]
    pub verified: bool,
    #[n(14)]
    pub time: u64,
    #[n(15)]
    pub operator: Operator,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Encode, Decode)]
pub struct IcpToEvmIdentifier(#[n(0)] pub LedgerBurnIndex, #[n(1)] pub ChainId);

impl IcpToEvmIdentifier {
    pub fn new(ledger_burn_index: LedgerBurnIndex, chain_id: ChainId) -> Self {
        Self(ledger_burn_index, chain_id)
    }
}

impl From<&AddIcpToEvmTx> for IcpToEvmIdentifier {
    fn from(value: &AddIcpToEvmTx) -> Self {
        let ledger_burn_index = LedgerBurnIndex::from(nat_to_u64(&value.native_ledger_burn_index));
        let chain_id = ChainId::from(&value.chain_id);
        Self::new(ledger_burn_index, chain_id)
    }
}

#[derive(
    Clone,
    PartialEq,
    Ord,
    Eq,
    PartialOrd,
    Debug,
    Hash,
    Encode,
    Decode,
    CandidType,
    Serialize,
    Deserialize,
)]
pub enum IcpToEvmStatus {
    #[n(0)]
    PendingVerification,
    #[n(1)]
    Accepted,
    #[n(2)]
    Created,
    #[n(3)]
    SignedTransaction,
    #[n(4)]
    ReplacedTransaction,
    #[n(5)]
    Reimbursed,
    #[n(6)]
    QuarantinedReimbursement,
    #[n(7)]
    Successful,
    #[n(8)]
    Failed,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Encode, Decode)]
pub struct IcpToEvmTx {
    #[n(0)]
    pub transaction_hash: Option<TransactionHash>,
    #[n(1)]
    pub native_ledger_burn_index: LedgerBurnIndex,
    #[n(2)]
    pub withdrawal_amount: Erc20TokenAmount,
    #[n(3)]
    pub actual_received: Option<Erc20TokenAmount>,
    #[n(4)]
    pub destination: Address,
    #[cbor(n(5), with = "crate::cbor::principal")]
    pub from: Principal,
    #[n(6)]
    pub chain_id: ChainId,
    #[cbor(n(7), with = "minicbor::bytes")]
    pub from_subaccount: Option<[u8; 32]>,
    #[n(8)]
    pub time: u64,
    #[n(9)]
    pub max_transaction_fee: Option<Erc20TokenAmount>,
    #[n(10)]
    pub effective_gas_price: Option<Erc20TokenAmount>,
    #[n(11)]
    pub gas_used: Option<Erc20TokenAmount>,
    #[n(12)]
    pub total_gas_spent: Option<Erc20TokenAmount>,
    #[n(13)]
    pub erc20_ledger_burn_index: Option<LedgerBurnIndex>,
    #[n(14)]
    pub erc20_contract_address: Address,
    #[cbor(n(15), with = "crate::cbor::principal::option")]
    pub icrc_ledger_id: Option<Principal>,
    #[n(16)]
    pub verified: bool,
    #[n(17)]
    pub status: IcpToEvmStatus,
    #[n(18)]
    pub operator: Operator,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Encode, Decode)]
pub struct Erc20Identifier(#[n(0)] pub Address, #[n(1)] pub ChainId);

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

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct EvmToken {
    #[n(0)]
    pub chain_id: ChainId,
    #[n(1)]
    pub erc20_contract_address: Address,
    #[n(2)]
    pub name: String,
    #[n(3)]
    pub decimals: u8,
    #[n(4)]
    pub symbol: String,
    #[n(5)]
    pub logo: String,
    #[n(6)]
    #[serde(default)]
    pub is_wrapped_icrc: bool,
    #[n(7)]
    pub cmc_id: Option<u64>,
    #[n(8)]
    pub usd_price: Option<String>,
    #[n(9)]
    pub volume_usd_24h: Option<String>,
}

#[derive(
    Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Encode, Decode, CandidType, Deserialize, Serialize,
)]
pub enum IcpTokenType {
    #[n(0)]
    ICRC1,
    #[n(1)]
    ICRC2,
    #[n(2)]
    ICRC3,
    #[n(3)]
    DIP20,
    #[n(4)]
    Other(#[n(0)] String),
}

#[derive(Clone, Eq, Ord, PartialOrd, Debug, Encode, Decode)]
pub struct IcpToken {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub ledger_id: Principal,
    #[n(1)]
    pub name: String,
    #[n(2)]
    pub decimals: u8,
    #[n(3)]
    pub symbol: String,
    #[n(4)]
    pub usd_price: String,
    #[n(5)]
    pub logo: String,
    #[n(6)]
    pub fee: Erc20TokenAmount,
    #[n(7)]
    pub token_type: IcpTokenType,
    #[n(8)]
    pub rank: Option<u32>,
    #[n(9)]
    pub listed_on_appic_dex: Option<bool>,
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

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Encode, Decode)]
pub struct BridgePair {
    #[n(0)]
    pub icp_token: IcpToken,
    #[n(1)]
    pub evm_token: EvmToken,
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Encode, Decode)]
pub enum Erc20TwinLedgerSuiteStatus {
    #[n(0)]
    PendingApproval,
    #[n(1)]
    Created,
    #[n(2)]
    Installed,
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Encode, Decode)]
pub enum Erc20TwinLedgerSuiteFee {
    #[n(0)]
    Icp(#[cbor(n(0), with = "crate::cbor::u128")] u128),
    #[n(1)]
    Appic(#[cbor(n(0), with = "crate::cbor::u128")] u128),
}

#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Debug,
    Deserialize,
    Serialize,
    Encode,
    Decode,
    CandidType,
)]
pub struct ChainId(#[n(0)] pub u64);

impl AsRef<u64> for ChainId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
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

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Encode, Decode)]
pub struct DexInfo {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub id: Principal,
    #[n(1)]
    pub last_observed_event: u64,
    #[n(2)]
    pub last_scraped_event: u64,
}
