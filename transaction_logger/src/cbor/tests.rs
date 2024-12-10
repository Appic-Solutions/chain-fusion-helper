use candid::{Nat, Principal};
use minicbor::{Decode, Encode};
use proptest::collection::vec as pvec;
use proptest::prelude::*;

pub fn check_roundtrip<T>(v: &T) -> Result<(), TestCaseError>
where
    for<'a> T: PartialEq + std::fmt::Debug + Encode<()> + Decode<'a, ()>,
{
    let mut buf = vec![];
    minicbor::encode(v, &mut buf).expect("encoding should succeed");
    let decoded = minicbor::decode(&buf).expect("decoding should succeed");
    prop_assert_eq!(v, &decoded);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct NatContainer {
    #[cbor(n(0), with = "crate::cbor::nat")]
    pub value: Nat,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct OptNatContainer {
    #[cbor(n(0), with = "crate::cbor::nat::option")]
    pub value: Option<Nat>,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct PrincipalContainer {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub value: Principal,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct OptPrincipalContainer {
    #[cbor(n(0), with = "crate::cbor::principal::option")]
    pub value: Option<Principal>,
}

proptest! {

    #[test]
    fn nat_encoding_roundtrip(n in any::<u128>()) {
        check_roundtrip(&NatContainer {
            value: Nat::from(n),
        })?;
    }

    #[test]
    fn opt_nat_encoding_roundtrip(n in proptest::option::of(any::<u128>())) {
        check_roundtrip(&OptNatContainer {
            value: n.map(Nat::from),
        })?;
    }

    #[test]
    fn principal_encoding_roundtrip(p in pvec(any::<u8>(), 0..30)) {
        check_roundtrip(&PrincipalContainer {
            value: Principal::from_slice(&p),
        })?;
    }

    #[test]
    fn opt_principal_encoding_roundtrip(p in proptest::option::of(pvec(any::<u8>(), 0..30))) {
        check_roundtrip(&OptPrincipalContainer {
            value: p.map(|principal| Principal::from_slice(&principal)),
        })?;
    }
}
