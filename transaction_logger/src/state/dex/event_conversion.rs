use std::any::Any;

use candid::{Nat, Principal};

use crate::{
    appic_dex_types::{CandidEvent, CandidEventType, CandidPoolId, CandidPositionKey, SwapType},
    state::{
        dex::types::{DexAction, PoolId, PositionKey, SwapType as CandidSwapType},
        nat_to_erc20_amount,
    },
};

impl From<CandidEvent> for DexAction {
    fn from(value: CandidEvent) -> Self {
        let timestamp = value.timestamp;

        match value.payload {
            crate::appic_dex_types::CandidEventType::Swap {
                principal: _,
                token_in,
                final_amount_in,
                final_amount_out,
                token_out,
                swap_type,
            } => Self::Swap {
                final_amount_in: nat_to_erc20_amount(final_amount_in),
                final_amount_out: nat_to_erc20_amount(final_amount_out),
                swap_type: swap_type.into(),
                timestamp,
                token_in,
                token_out,
            },
            crate::appic_dex_types::CandidEventType::CreatedPool {
                token0,
                token1,
                pool_fee,
            } => Self::CreatedPool {
                token0,
                token1,
                pool_fee: pool_fee.0.try_into().unwrap(),
                timestamp,
            },
            crate::appic_dex_types::CandidEventType::BurntPosition {
                amount0_received,
                principal: _,
                burnt_position,
                liquidity,
                amount1_received,
            } => Self::BurntPosition {
                burnt_position: burnt_position.into(),
                liquidity: liquidity.0.try_into().unwrap(),
                amount0_received: nat_to_erc20_amount(amount0_received),
                amount1_received: nat_to_erc20_amount(amount1_received),
                timestamp,
            },
            crate::appic_dex_types::CandidEventType::IncreasedLiquidity {
                principal: _,
                amount0_paid,
                liquidity_delta,
                amount1_paid,
                modified_position,
            } => Self::IncreasedLiquidity {
                modified_position: modified_position.into(),
                liquidity_delta: liquidity_delta.0.try_into().unwrap(),
                amount0_paid: nat_to_erc20_amount(amount0_paid),
                amount1_paid: nat_to_erc20_amount(amount1_paid),
                timestamp,
            },
            crate::appic_dex_types::CandidEventType::CollectedFees {
                principal: _,
                amount1_collected,
                position,
                amount0_collected,
            } => Self::CollectedFees {
                position: position.into(),
                amount0_collected: nat_to_erc20_amount(amount0_collected),
                amount1_collected: nat_to_erc20_amount(amount1_collected),
                timestamp,
            },
            crate::appic_dex_types::CandidEventType::DecreasedLiquidity {
                amount0_received,
                principal: _,
                liquidity_delta,
                amount1_received,
                modified_position,
            } => Self::DecreasedLiquidity {
                modified_position: modified_position.into(),
                liquidity_delta: liquidity_delta.0.try_into().unwrap(),
                amount0_received: nat_to_erc20_amount(amount0_received),
                amount1_received: nat_to_erc20_amount(amount1_received),
                timestamp,
            },
            crate::appic_dex_types::CandidEventType::MintedPosition {
                principal: _,
                amount0_paid,
                liquidity,
                created_position,
                amount1_paid,
            } => Self::MintedPosition {
                created_position: created_position.into(),
                liquidity: liquidity.0.try_into().unwrap(),
                amount0_paid: nat_to_erc20_amount(amount0_paid),
                amount1_paid: nat_to_erc20_amount(amount1_paid),
                timestamp,
            },
        }
    }
}

impl From<(Principal, DexAction)> for CandidEvent {
    fn from(value: (Principal, DexAction)) -> Self {
        match value.1 {
            DexAction::CreatedPool {
                token0,
                token1,
                pool_fee,
                timestamp,
            } => CandidEvent {
                timestamp,
                payload: CandidEventType::CreatedPool {
                    token0,
                    token1,
                    pool_fee: Nat::from(pool_fee),
                },
            },
            DexAction::MintedPosition {
                created_position,
                liquidity,
                amount0_paid,
                amount1_paid,
                timestamp,
            } => CandidEvent {
                timestamp,
                payload: CandidEventType::MintedPosition {
                    principal: value.0,
                    amount0_paid: amount0_paid.into(),
                    liquidity: liquidity.into(),
                    created_position: created_position.into(),
                    amount1_paid: amount1_paid.into(),
                },
            },
            DexAction::IncreasedLiquidity {
                modified_position,
                liquidity_delta,
                amount0_paid,
                amount1_paid,
                timestamp,
            } => CandidEvent {
                timestamp,
                payload: CandidEventType::IncreasedLiquidity {
                    principal: value.0,
                    amount0_paid: amount0_paid.into(),
                    liquidity_delta: liquidity_delta.into(),
                    amount1_paid: amount1_paid.into(),
                    modified_position: modified_position.into(),
                },
            },
            DexAction::BurntPosition {
                burnt_position,
                liquidity,
                amount0_received,
                amount1_received,
                timestamp,
            } => CandidEvent {
                timestamp,
                payload: CandidEventType::BurntPosition {
                    amount0_received: amount0_received.into(),
                    principal: value.0,
                    burnt_position: burnt_position.into(),
                    liquidity: liquidity.into(),
                    amount1_received: amount1_received.into(),
                },
            },
            DexAction::DecreasedLiquidity {
                modified_position,
                liquidity_delta,
                amount0_received,
                amount1_received,
                timestamp,
            } => CandidEvent {
                timestamp,
                payload: CandidEventType::DecreasedLiquidity {
                    amount0_received: amount0_received.into(),
                    principal: value.0,
                    liquidity_delta: liquidity_delta.into(),
                    amount1_received: amount1_received.into(),
                    modified_position: modified_position.into(),
                },
            },
            DexAction::CollectedFees {
                position,
                amount0_collected,
                amount1_collected,
                timestamp,
            } => CandidEvent {
                timestamp,
                payload: CandidEventType::CollectedFees {
                    principal: value.0,
                    amount1_collected: amount1_collected.into(),
                    position: position.into(),
                    amount0_collected: amount0_collected.into(),
                },
            },
            DexAction::Swap {
                final_amount_in,
                final_amount_out,
                swap_type,
                timestamp,
                token_in,
                token_out,
            } => CandidEvent {
                timestamp,
                payload: CandidEventType::Swap {
                    principal: value.0,
                    token_in,
                    final_amount_in: final_amount_in.into(),
                    final_amount_out: final_amount_out.into(),
                    token_out,
                    swap_type: swap_type.into(),
                },
            },
        }
    }
}

// safe since the event are coming from dex canister and they're already validated
impl From<CandidPositionKey> for PositionKey {
    fn from(value: CandidPositionKey) -> Self {
        let pool_id = PoolId::from(value.pool);

        let tick_lower: i32 = value.tick_lower.0.try_into().unwrap();

        let tick_upper: i32 = value.tick_upper.0.try_into().unwrap();

        PositionKey {
            owner: value.owner,
            pool_id,
            tick_lower,
            tick_upper,
        }
    }
}

impl From<PositionKey> for CandidPositionKey {
    fn from(value: PositionKey) -> Self {
        Self {
            owner: value.owner,
            pool: value.pool_id.into(),
            tick_lower: value.tick_lower.into(),
            tick_upper: value.tick_upper.into(),
        }
    }
}

// safe since the event are coming from dex canister and they're already validated
impl From<CandidPoolId> for PoolId {
    fn from(value: CandidPoolId) -> Self {
        let fee: u32 = value.fee.0.try_into().unwrap();

        PoolId {
            token0: value.token0,
            token1: value.token1,
            fee,
        }
    }
}

impl From<PoolId> for CandidPoolId {
    fn from(value: PoolId) -> CandidPoolId {
        let fee: Nat = value.fee.into();

        CandidPoolId {
            token0: value.token0,
            token1: value.token1,
            fee,
        }
    }
}

impl From<CandidSwapType> for SwapType {
    fn from(value: CandidSwapType) -> Self {
        match value {
            CandidSwapType::ExactOutput(pool_ids) => {
                Self::ExactOutput(pool_ids.into_iter().map(|pool| pool.into()).collect())
            }
            CandidSwapType::ExactInput(pool_ids) => {
                Self::ExactInput(pool_ids.into_iter().map(|pool| pool.into()).collect())
            }

            CandidSwapType::ExactOutputSingle(pool_id) => Self::ExactOutputSingle(pool_id.into()),
            CandidSwapType::ExactInputSingle(pool_id) => Self::ExactInputSingle(pool_id.into()),
        }
    }
}

impl From<SwapType> for CandidSwapType {
    fn from(value: SwapType) -> Self {
        match value {
            SwapType::ExactOutput(pool_ids) => {
                Self::ExactOutput(pool_ids.into_iter().map(|pool| pool.into()).collect())
            }
            SwapType::ExactInput(pool_ids) => {
                Self::ExactInput(pool_ids.into_iter().map(|pool| pool.into()).collect())
            }

            SwapType::ExactOutputSingle(pool_id) => Self::ExactOutputSingle(pool_id.into()),
            SwapType::ExactInputSingle(pool_id) => Self::ExactInputSingle(pool_id.into()),
        }
    }
}
