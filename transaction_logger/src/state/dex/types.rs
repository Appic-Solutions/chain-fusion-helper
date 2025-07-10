use candid::Principal;
use minicbor::{Decode, Encode};

use crate::numeric::Erc20TokenAmount;

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PoolId {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub token0: Principal, // Token0 identifier
    #[cbor(n(1), with = "crate::cbor::principal")]
    pub token1: Principal, // Token1 identifier
    #[n(2)]
    pub fee: u32, // Fee tier (e.g., 500 for 0.05%)
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PositionKey {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub owner: Principal,
    #[n(1)]
    pub pool_id: PoolId,
    #[n(2)]
    pub tick_lower: i32,
    #[n(3)]
    pub tick_upper: i32,
}

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone)]
pub enum SwapType {
    #[n(0)]
    ExactOutput(#[n(0)] Vec<PoolId>),
    #[n(1)]
    ExactInput(#[n(0)] Vec<PoolId>),
    #[n(2)]
    ExactOutputSingle(#[n(0)] PoolId),
    #[n(3)]
    ExactInputSingle(#[n(0)] PoolId),
}

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone)]
pub struct UserDexActions(#[n(0)] pub Vec<DexAction>);

/// The event describing the  minter state transition.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub enum DexAction {
    #[n(0)]
    CreatedPool {
        #[cbor(n(0), with = "crate::cbor::principal")]
        token0: Principal,
        #[cbor(n(1), with = "crate::cbor::principal")]
        token1: Principal,
        #[n(2)]
        pool_fee: u32,
        #[n(3)]
        timestamp: u64,
    },
    #[n(1)]
    MintedPosition {
        #[n(0)]
        created_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity: u128,
        #[n(2)]
        amount0_paid: Erc20TokenAmount,
        #[n(3)]
        amount1_paid: Erc20TokenAmount,
        #[n(4)]
        timestamp: u64,
    },
    #[n(2)]
    IncreasedLiquidity {
        #[n(0)]
        modified_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity_delta: u128,
        #[n(2)]
        amount0_paid: Erc20TokenAmount,
        #[n(3)]
        amount1_paid: Erc20TokenAmount,
        #[n(4)]
        timestamp: u64,
    },
    #[n(3)]
    BurntPosition {
        #[n(0)]
        burnt_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity: u128,
        #[n(2)]
        amount0_received: Erc20TokenAmount,
        #[n(3)]
        amount1_received: Erc20TokenAmount,
        #[n(4)]
        timestamp: u64,
    },
    #[n(4)]
    DecreasedLiquidity {
        #[n(0)]
        modified_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity_delta: u128,
        #[n(2)]
        amount0_received: Erc20TokenAmount,
        #[n(3)]
        amount1_received: Erc20TokenAmount,
        #[n(4)]
        timestamp: u64,
    },
    #[n(5)]
    CollectedFees {
        #[n(0)]
        position: PositionKey,
        #[n(1)]
        amount0_collected: Erc20TokenAmount,
        #[n(2)]
        amount1_collected: Erc20TokenAmount,
        #[n(3)]
        timestamp: u64,
    },
    #[n(6)]
    Swap {
        #[n(0)]
        final_amount_in: Erc20TokenAmount,
        #[n(1)]
        final_amount_out: Erc20TokenAmount,
        #[n(2)]
        swap_type: SwapType,
        #[n(3)]
        timestamp: u64,
        #[cbor(n(4), with = "crate::cbor::principal")]
        token_in: Principal,
        #[cbor(n(5), with = "crate::cbor::principal")]
        token_out: Principal,
    },
}
