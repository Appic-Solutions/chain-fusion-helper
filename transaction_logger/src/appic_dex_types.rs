// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;

use crate::cbor::principal;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct CandidPoolId {
    pub fee: candid::Nat,
    pub token0: Principal,
    pub token1: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct BurnPositionArgs {
    pub amount1_min: candid::Nat,
    pub pool: CandidPoolId,
    pub amount0_min: candid::Nat,
    pub tick_lower: candid::Int,
    pub tick_upper: candid::Int,
}

#[derive(CandidType, Deserialize)]
pub enum WithdrawError {
    FeeUnknown,
    TemporarilyUnavailable(String),
    InvalidDestination(String),
    InsufficientAllowance { allowance: candid::Nat },
    InsufficientBalance { balance: candid::Nat },
    AmountTooLow { min_withdrawal_amount: candid::Nat },
    LockedPrincipal,
    AmountOverflow,
}

#[derive(CandidType, Deserialize)]
pub enum BurnPositionError {
    PositionNotFound,
    InvalidAmount,
    InvalidPoolFee,
    PoolNotInitialized,
    InsufficientBalance,
    LiquidityOverflow,
    FeeOverflow,
    SlippageFailed,
    BurntPositionWithdrawalFailed(WithdrawError),
    InvalidTick,
    LockedPrincipal,
    AmountOverflow,
}

#[derive(CandidType, Deserialize)]
pub enum Result_ {
    Ok,
    Err(BurnPositionError),
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CandidPositionKey {
    pub owner: Principal,
    pub pool: CandidPoolId,
    pub tick_lower: candid::Int,
    pub tick_upper: candid::Int,
}

#[derive(CandidType, Deserialize)]
pub struct CollectFeesSuccess {
    pub token0_collected: candid::Nat,
    pub token1_collected: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub enum CollectFeesError {
    PositionNotFound,
    FeeOverflow,
    LockedPrincipal,
    CollectedFeesWithdrawalFailed(WithdrawError),
    NoFeeToCollect,
}

#[derive(CandidType, Deserialize)]
pub enum Result1 {
    Ok(CollectFeesSuccess),
    Err(CollectFeesError),
}

#[derive(CandidType, Deserialize)]
pub struct CreatePoolArgs {
    pub fee: candid::Nat,
    pub sqrt_price_x96: candid::Nat,
    pub token_a: Principal,
    pub token_b: Principal,
}

#[derive(CandidType, Deserialize)]
pub enum CreatePoolError {
    InvalidSqrtPriceX96,
    InvalidFeeAmount,
    DuplicatedTokens,
    InvalidToken(Principal),
    PoolAlreadyExists,
}

#[derive(CandidType, Deserialize)]
pub enum Result2 {
    Ok(CandidPoolId),
    Err(CreatePoolError),
}

#[derive(CandidType, Deserialize)]
pub struct DecreaseLiquidityArgs {
    pub amount1_min: candid::Nat,
    pub pool: CandidPoolId,
    pub liquidity: candid::Nat,
    pub amount0_min: candid::Nat,
    pub tick_lower: candid::Int,
    pub tick_upper: candid::Int,
}

#[derive(CandidType, Deserialize)]
pub enum DecreaseLiquidityError {
    PositionNotFound,
    InvalidAmount,
    InvalidPoolFee,
    PoolNotInitialized,
    InsufficientBalance,
    LiquidityOverflow,
    FeeOverflow,
    SlippageFailed,
    InvalidTick,
    InvalidLiquidity,
    LockedPrincipal,
    AmountOverflow,
    DecreasedPositionWithdrawalFailed(WithdrawError),
}

#[derive(CandidType, Deserialize)]
pub enum Result3 {
    Ok,
    Err(DecreaseLiquidityError),
}

#[derive(CandidType, Deserialize)]
pub struct DepositArgs {
    pub token: Principal,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub amount: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub enum DepositError {
    TemporarilyUnavailable(String),
    InvalidDestination(String),
    InsufficientAllowance { allowance: candid::Nat },
    AmountTooLow { min_withdrawal_amount: candid::Nat },
    LockedPrincipal,
    AmountOverflow,
    InsufficientFunds { balance: candid::Nat },
}

#[derive(CandidType, Deserialize)]
pub enum Result4 {
    Ok,
    Err(DepositError),
}

#[derive(CandidType, Deserialize)]
pub struct CandidTickInfo {
    pub fee_growth_outside_1_x128: candid::Nat,
    pub liquidity_gross: candid::Nat,
    pub tick: candid::Int,
    pub liquidity_net: candid::Int,
    pub fee_growth_outside_0_x128: candid::Nat,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum SwapType {
    ExactOutput(Vec<CandidPoolId>),
    ExactInput(Vec<CandidPoolId>),
    ExactOutputSingle(CandidPoolId),
    ExactInputSingle(CandidPoolId),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum CandidEventType {
    Swap {
        principal: Principal,
        token_in: Principal,
        final_amount_in: candid::Nat,
        final_amount_out: candid::Nat,
        token_out: Principal,
        swap_type: SwapType,
    },
    CreatedPool {
        token0: Principal,
        token1: Principal,
        pool_fee: candid::Nat,
    },
    BurntPosition {
        amount0_received: candid::Nat,
        principal: Principal,
        burnt_position: CandidPositionKey,
        liquidity: candid::Nat,
        amount1_received: candid::Nat,
    },
    IncreasedLiquidity {
        principal: Principal,
        amount0_paid: candid::Nat,
        liquidity_delta: candid::Nat,
        amount1_paid: candid::Nat,
        modified_position: CandidPositionKey,
    },
    CollectedFees {
        principal: Principal,
        amount1_collected: candid::Nat,
        position: CandidPositionKey,
        amount0_collected: candid::Nat,
    },
    DecreasedLiquidity {
        amount0_received: candid::Nat,
        principal: Principal,
        liquidity_delta: candid::Nat,
        amount1_received: candid::Nat,
        modified_position: CandidPositionKey,
    },
    MintedPosition {
        principal: Principal,
        amount0_paid: candid::Nat,
        liquidity: candid::Nat,
        created_position: CandidPositionKey,
        amount1_paid: candid::Nat,
    },
}

impl CandidEventType {
    pub fn get_principal(&self) -> Principal {
        match self {
            CandidEventType::Swap {
                principal,
                token_in,
                final_amount_in,
                final_amount_out,
                token_out,
                swap_type,
            } => *principal,
            CandidEventType::CreatedPool {
                token0,
                token1,
                pool_fee,
            } => Principal::anonymous(),
            CandidEventType::BurntPosition {
                amount0_received,
                principal,
                burnt_position,
                liquidity,
                amount1_received,
            } => *principal,
            CandidEventType::IncreasedLiquidity {
                principal,
                amount0_paid,
                liquidity_delta,
                amount1_paid,
                modified_position,
            } => *principal,
            CandidEventType::CollectedFees {
                principal,
                amount1_collected,
                position,
                amount0_collected,
            } => *principal,
            CandidEventType::DecreasedLiquidity {
                amount0_received,
                principal,
                liquidity_delta,
                amount1_received,
                modified_position,
            } => *principal,
            CandidEventType::MintedPosition {
                principal,
                amount0_paid,
                liquidity,
                created_position,
                amount1_paid,
            } => *principal,
        }
    }
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CandidEvent {
    pub timestamp: u64,
    pub payload: CandidEventType,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GetEventsResult {
    pub total_event_count: u64,
    pub events: Vec<CandidEvent>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CandidPoolState {
    pub sqrt_price_x96: candid::Nat,
    pub pool_reserves0: candid::Nat,
    pub pool_reserves1: candid::Nat,
    pub fee_protocol: candid::Nat,
    pub token0_transfer_fee: candid::Nat,
    pub swap_volume1_all_time: candid::Nat,
    pub fee_growth_global_1_x128: candid::Nat,
    pub tick: candid::Int,
    pub liquidity: candid::Nat,
    pub generated_swap_fee0: candid::Nat,
    pub generated_swap_fee1: candid::Nat,
    pub swap_volume0_all_time: candid::Nat,
    pub fee_growth_global_0_x128: candid::Nat,
    pub max_liquidity_per_tick: candid::Nat,
    pub token1_transfer_fee: candid::Nat,
    pub tick_spacing: candid::Int,
}

#[derive(CandidType, Deserialize)]
pub struct CandidHistoryBucket {
    pub token0_reserves: candid::Nat,
    pub end_timestamp: u64,
    pub swap_volume_token0_during_bucket: candid::Nat,
    pub fee_generated_token1_during_bucket: candid::Nat,
    pub fee_generated_token0_start: candid::Nat,
    pub start_timestamp: u64,
    pub inrange_liquidity: candid::Nat,
    pub fee_generated_token1_start: candid::Nat,
    pub swap_volume_token0_start: candid::Nat,
    pub swap_volume_token1_start: candid::Nat,
    pub fee_generated_token0_during_bucket: candid::Nat,
    pub last_sqrtx96_price: candid::Nat,
    pub swap_volume_token1_during_bucket: candid::Nat,
    pub token1_reserves: candid::Nat,
    pub active_tick: candid::Int,
}

#[derive(CandidType, Deserialize)]
pub struct CandidPoolHistory {
    pub hourly_frame: Vec<CandidHistoryBucket>,
    pub monthly_frame: Vec<CandidHistoryBucket>,
    pub yearly_frame: Vec<CandidHistoryBucket>,
    pub daily_frame: Vec<CandidHistoryBucket>,
}

#[derive(CandidType, Deserialize)]
pub struct CandidPositionInfo {
    pub fees_token0_owed: candid::Nat,
    pub fee_growth_inside_1_last_x128: candid::Nat,
    pub liquidity: candid::Nat,
    pub fees_token1_owed: candid::Nat,
    pub fee_growth_inside_0_last_x128: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub struct IncreaseLiquidityArgs {
    pub amount1_max: candid::Nat,
    pub pool: CandidPoolId,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub amount0_max: candid::Nat,
    pub tick_lower: candid::Int,
    pub tick_upper: candid::Int,
}

#[derive(CandidType, Deserialize)]
pub enum IncreaseLiquidityError {
    DepositError(DepositError),
    TickNotAlignedWithTickSpacing,
    InvalidAmount,
    InvalidPoolFee,
    PoolNotInitialized,
    InsufficientBalance,
    LiquidityOverflow,
    FeeOverflow,
    SlippageFailed,
    InvalidTick,
    PositionDoesNotExist,
    LockedPrincipal,
    AmountOverflow,
}

#[derive(CandidType, Deserialize)]
pub enum Result5 {
    Ok(candid::Nat),
    Err(IncreaseLiquidityError),
}

#[derive(CandidType, Deserialize)]
pub struct MintPositionArgs {
    pub amount1_max: candid::Nat,
    pub pool: CandidPoolId,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub amount0_max: candid::Nat,
    pub tick_lower: candid::Int,
    pub tick_upper: candid::Int,
}

#[derive(CandidType, Deserialize)]
pub enum MintPositionError {
    DepositError(DepositError),
    TickNotAlignedWithTickSpacing,
    InvalidAmount,
    InvalidPoolFee,
    PoolNotInitialized,
    InsufficientBalance,
    LiquidityOverflow,
    FeeOverflow,
    SlippageFailed,
    PositionAlreadyExists,
    InvalidTick,
    LockedPrincipal,
    AmountOverflow,
}

#[derive(CandidType, Deserialize)]
pub enum Result6 {
    Ok(candid::Nat),
    Err(MintPositionError),
}

#[derive(CandidType, Deserialize)]
pub struct CandidPathKey {
    pub fee: candid::Nat,
    pub intermediary_token: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct QuoteExactParams {
    pub path: Vec<CandidPathKey>,
    pub exact_token: Principal,
    pub exact_amount: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub struct QuoteExactSingleParams {
    pub zero_for_one: bool,
    pub pool_id: CandidPoolId,
    pub exact_amount: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub enum QuoteArgs {
    QuoteExactOutput(QuoteExactParams),
    QuoteExactOutputSingleParams(QuoteExactSingleParams),
    QuoteExactInputParams(QuoteExactParams),
    QuoteExactInputSingleParams(QuoteExactSingleParams),
}

#[derive(CandidType, Deserialize)]
pub enum QuoteError {
    InvalidAmount,
    PoolNotInitialized,
    InvalidFee,
    PriceLimitOutOfBounds,
    InvalidPathLength,
    IlliquidPool,
    PriceLimitAlreadyExceeded,
    InvalidFeeForExactOutput,
    CalculationOverflow,
}

#[derive(CandidType, Deserialize)]
pub enum Result7 {
    Ok(candid::Nat),
    Err(QuoteError),
}

#[derive(CandidType, Deserialize)]
pub struct ExactOutputParams {
    pub amount_in_maximum: candid::Nat,
    pub path: Vec<CandidPathKey>,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub amount_out: candid::Nat,
    pub token_out: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct ExactInputParams {
    pub token_in: Principal,
    pub path: Vec<CandidPathKey>,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub amount_out_minimum: candid::Nat,
    pub amount_in: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub struct ExactOutputSingleParams {
    pub amount_in_maximum: candid::Nat,
    pub zero_for_one: bool,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub amount_out: candid::Nat,
    pub pool_id: CandidPoolId,
}

#[derive(CandidType, Deserialize)]
pub struct ExactInputSingleParams {
    pub zero_for_one: bool,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub amount_out_minimum: candid::Nat,
    pub amount_in: candid::Nat,
    pub pool_id: CandidPoolId,
}

#[derive(CandidType, Deserialize)]
pub enum SwapArgs {
    ExactOutput(ExactOutputParams),
    ExactInput(ExactInputParams),
    ExactOutputSingle(ExactOutputSingleParams),
    ExactInputSingle(ExactInputSingleParams),
}

#[derive(CandidType, Deserialize)]
pub struct CandidSwapSuccess {
    pub amount_out: candid::Nat,
    pub amount_in: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub enum SwapFailedReason {
    TooMuchRequested,
    InvalidAmount,
    PoolNotInitialized,
    InsufficientBalance,
    PriceLimitOutOfBounds,
    BalanceOverflow,
    TooLittleReceived,
    NoInRangeLiquidity,
    PriceLimitAlreadyExceeded,
    InvalidFeeForExactOutput,
    CalculationOverflow,
}

#[derive(CandidType, Deserialize)]
pub enum SwapError {
    FailedToWithdraw {
        amount_out: candid::Nat,
        amount_in: candid::Nat,
        reason: WithdrawError,
    },
    InvalidAmountOut,
    DepositError(DepositError),
    InvalidAmountIn,
    InvalidAmountInMaximum,
    InvalidAmountOutMinimum,
    InvalidPoolFee,
    PoolNotInitialized,
    PathLengthTooSmall {
        minimum: u8,
        received: u8,
    },
    PathDuplicated,
    PathLengthTooBig {
        maximum: u8,
        received: u8,
    },
    LockedPrincipal,
    NoInRangeLiquidity,
    SwapFailedRefunded {
        refund_error: Option<WithdrawError>,
        refund_amount: Option<candid::Nat>,
        failed_reason: SwapFailedReason,
    },
}

#[derive(CandidType, Deserialize)]
pub enum Result8 {
    Ok(CandidSwapSuccess),
    Err(SwapError),
}

#[derive(CandidType, Deserialize)]
pub struct UserBalanceArgs {
    pub token: Principal,
    pub user: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct Balance {
    pub token: Principal,
    pub amount: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub enum Result9 {
    Ok(candid::Nat),
    Err(WithdrawError),
}

pub struct Service(pub Principal);
impl Service {
    pub async fn burn(&self, arg0: BurnPositionArgs) -> Result<(Result_,)> {
        ic_cdk::call(self.0, "burn", (arg0,)).await
    }
    pub async fn collect_fees(&self, arg0: CandidPositionKey) -> Result<(Result1,)> {
        ic_cdk::call(self.0, "collect_fees", (arg0,)).await
    }
    pub async fn create_pool(&self, arg0: CreatePoolArgs) -> Result<(Result2,)> {
        ic_cdk::call(self.0, "create_pool", (arg0,)).await
    }
    pub async fn decrease_liquidity(&self, arg0: DecreaseLiquidityArgs) -> Result<(Result3,)> {
        ic_cdk::call(self.0, "decrease_liquidity", (arg0,)).await
    }
    pub async fn deposit(&self, arg0: DepositArgs) -> Result<(Result4,)> {
        ic_cdk::call(self.0, "deposit", (arg0,)).await
    }
    pub async fn get_active_ticks(&self, arg0: CandidPoolId) -> Result<(Vec<CandidTickInfo>,)> {
        ic_cdk::call(self.0, "get_active_ticks", (arg0,)).await
    }
    pub async fn get_events(&self, arg0: GetEventsArg) -> Result<(GetEventsResult,)> {
        ic_cdk::call(self.0, "get_events", (arg0,)).await
    }
    pub async fn get_pool(&self, arg0: CandidPoolId) -> Result<(Option<CandidPoolState>,)> {
        ic_cdk::call(self.0, "get_pool", (arg0,)).await
    }
    pub async fn get_pool_history(
        &self,
        arg0: CandidPoolId,
    ) -> Result<(Option<CandidPoolHistory>,)> {
        ic_cdk::call(self.0, "get_pool_history", (arg0,)).await
    }
    pub async fn get_pools(&self) -> Result<(Vec<(CandidPoolId, CandidPoolState)>,)> {
        ic_cdk::call(self.0, "get_pools", ()).await
    }
    pub async fn get_position(
        &self,
        arg0: CandidPositionKey,
    ) -> Result<(Option<CandidPositionInfo>,)> {
        ic_cdk::call(self.0, "get_position", (arg0,)).await
    }
    pub async fn get_positions_by_owner(
        &self,
        arg0: Principal,
    ) -> Result<(Vec<(CandidPositionKey, CandidPositionInfo)>,)> {
        ic_cdk::call(self.0, "get_positions_by_owner", (arg0,)).await
    }
    pub async fn increase_liquidity(&self, arg0: IncreaseLiquidityArgs) -> Result<(Result5,)> {
        ic_cdk::call(self.0, "increase_liquidity", (arg0,)).await
    }
    pub async fn mint_position(&self, arg0: MintPositionArgs) -> Result<(Result6,)> {
        ic_cdk::call(self.0, "mint_position", (arg0,)).await
    }
    pub async fn quote(&self, arg0: QuoteArgs) -> Result<(Result7,)> {
        ic_cdk::call(self.0, "quote", (arg0,)).await
    }
    pub async fn swap(&self, arg0: SwapArgs) -> Result<(Result8,)> {
        ic_cdk::call(self.0, "swap", (arg0,)).await
    }
    pub async fn user_balance(&self, arg0: UserBalanceArgs) -> Result<(candid::Nat,)> {
        ic_cdk::call(self.0, "user_balance", (arg0,)).await
    }
    pub async fn user_balances(&self, arg0: Principal) -> Result<(Vec<Balance>,)> {
        ic_cdk::call(self.0, "user_balances", (arg0,)).await
    }
    pub async fn withdraw(&self, arg0: Balance) -> Result<(Result9,)> {
        ic_cdk::call(self.0, "withdraw", (arg0,)).await
    }
}
