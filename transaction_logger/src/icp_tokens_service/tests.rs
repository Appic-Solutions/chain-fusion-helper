use candid::{Int, Nat};

use super::*;

#[test]
fn test_claculate_usd_price_based_on_ck_usdc() {
    // Setup pool_id
    let pool_id = CandidPoolId {
        token0: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        token1: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap(),
        fee: Nat::from(3000_u64),
    };

    // Setup pool_state
    let pool_state = CandidPoolState {
        sqrt_price_x96: Nat::from(17_398_315_620_450_599_914_070_061_634_u128),
        pool_reserves0: Nat::from(1_497_894_223_u128),
        pool_reserves1: Nat::from(72_233_125_u128),
        fee_protocol: Nat::from(0_u128),
        token0_transfer_fee: Nat::from(10_000_u128),
        swap_volume1_all_time: Nat::from(4_980_000_u128),
        fee_growth_global_1_x128: Nat::from(7_735_301_346_986_762_823_205_250_238_450_708_u128),
        tick: Int::from(-30_321),
        liquidity: Nat::from(328_934_000_u128),
        generated_swap_fee0: Nat::from(299_941_u128),
        generated_swap_fee1: Nat::from(14_941_u128),
        swap_volume0_all_time: Nat::from(99_980_000_u128),
        fee_growth_global_0_x128: Nat::from(155_286_394_573_091_267_515_896_256_058_573_335_u128),
        max_liquidity_per_tick: Nat::from(11_505_743_598_341_114_571_880_798_222_544_994_u128),
        token1_transfer_fee: Nat::from(10_000_u128),
        tick_spacing: Int::from(60_u128),
    };

    // Setup other parameters
    let other_token = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let ck_usdc_ledger_id = Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap();

    let mut decimals_cache = HashMap::new();
    decimals_cache.insert(ck_usdc_ledger_id, 6); // ckUSDC typically has 6 decimals
    decimals_cache.insert(other_token, 8); // Many tokens use 18 decimals

    // Calculate USD price
    let usd_price = claculate_usd_price_based_on_ck_usdc(
        &pool_id,
        pool_state,
        &other_token,
        &ck_usdc_ledger_id,
        &decimals_cache,
    );

    println!("{:?}", U256::ONE << 96);

    println!("{:?}", usd_price);

    // Expected value: p = 219^2 = 47961, usd_price = 47961 * 10^12 = 4.7961e16
    let expected_usd_price = 4.822311302083245;

    //Assert with small epsilon due to floating-point arithmetic
    assert!(
        (usd_price - expected_usd_price).abs() < 1e-6,
        "USD price mismatch: expected {}, got {}",
        expected_usd_price,
        usd_price
    );
}
