use crate::checked_amount::CheckedAmountOf;

pub enum WeiTag {}
pub type Wei = CheckedAmountOf<WeiTag>;

pub enum Erc20Tag {}
pub type Erc20Value = CheckedAmountOf<Erc20Tag>;

/// Amount of Erc20 twin token using their smallest denomination.
pub enum Erc20TokenAmountTag {}
pub type Erc20TokenAmount = CheckedAmountOf<Erc20TokenAmountTag>;

pub enum WeiPerGasUnit {}
pub type WeiPerGas = CheckedAmountOf<WeiPerGasUnit>;

pub enum BlockNumberTag {}
pub type BlockNumber = CheckedAmountOf<BlockNumberTag>;

pub enum BurnIndexTag {}
pub type LedgerBurnIndex = u64;

pub enum MintIndexTag {}
pub type LedgerMintIndex = u64;
