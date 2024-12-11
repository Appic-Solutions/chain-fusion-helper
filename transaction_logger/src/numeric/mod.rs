use crate::checked_amount::CheckedAmountOf;
use phantom_newtype::Id;

pub enum WeiTag {}
pub type Wei = CheckedAmountOf<WeiTag>;

pub enum Erc20Tag {}
pub type Erc20Value = CheckedAmountOf<Erc20Tag>;

/// Amount of Erc20 tiwn token using their smallest denomination.
pub enum Erc20TokenAmountTag {}
pub type Erc20TokenAmount = CheckedAmountOf<Erc20TokenAmountTag>;

pub enum WeiPerGasUnit {}
pub type WeiPerGas = CheckedAmountOf<WeiPerGasUnit>;

pub enum BlockNumberTag {}
pub type BlockNumber = CheckedAmountOf<BlockNumberTag>;

pub enum BurnIndexTag {}
pub type LedgerBurnIndex = Id<BurnIndexTag, u64>;
