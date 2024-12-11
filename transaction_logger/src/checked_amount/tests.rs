use crate::checked_amount::CheckedAmountOf;

mod checked_div_ceil {
    use super::Amount;
    use proptest::prelude::any;
    use proptest::proptest;

    proptest! {
        #[test]
        fn should_be_zero_when_dividend_is_zero(divisor in 1_u128..=u128::MAX) {
            assert_eq!(Amount::ZERO, Amount::ZERO.checked_div_ceil(divisor).unwrap());
        }
    }

    proptest! {
        #[test]
        fn should_be_none_when_divisor_is_zero(amount in any::<u128>()) {
            assert_eq!(None, Amount::from(amount).checked_div_ceil(0_u8));
        }
    }

    proptest! {
    #[test]
    fn should_increment_quotient_of_floor_division_when_not_multiple_of_divisor(divisor in 1_u128..=u128::MAX) {
        let large_prime_number = Amount::from_str_hex(
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F",
        )
        .expect("valid u256 since this is the p parameter of ECDSA Secp256k1 curve");

        let actual_quotient = large_prime_number.checked_div_ceil(divisor).unwrap();

        let expected_quotient = large_prime_number.0 / divisor + 1;
        assert_eq!(expected_quotient, actual_quotient.0);
    }
    }
}

enum Unit {}
type Amount = CheckedAmountOf<Unit>;
