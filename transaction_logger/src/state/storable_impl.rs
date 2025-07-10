use ic_stable_structures::{storable::Bound, storable::Storable};
use std::borrow::Cow;

use crate::state::{
    dex::types::{DexAction, SwapType, UserDexActions},
    types::*,
};

macro_rules! impl_storable_minicbor {
    ($type:ty ) => {
        impl Storable for $type {
            fn to_bytes(&self) -> Cow<[u8]> {
                let mut buf = Vec::new();
                minicbor::encode(self, &mut buf).expect("minicbor encoding should always succeed");
                Cow::Owned(buf)
            }

            fn from_bytes(bytes: Cow<[u8]>) -> Self {
                minicbor::decode(bytes.as_ref()).unwrap_or_else(|e| {
                    panic!(
                        "failed to decode minicbor bytes {}: {}",
                        hex::encode(&bytes),
                        e
                    )
                })
            }
            const BOUND: Bound = Bound::Unbounded;
        }
    };
}

//// Apply to your types
impl_storable_minicbor!(Minter);
impl_storable_minicbor!(MinterKey);
impl_storable_minicbor!(Erc20Identifier);
impl_storable_minicbor!(EvmToIcpTxIdentifier);
impl_storable_minicbor!(IcpToEvmIdentifier);
impl_storable_minicbor!(EvmToIcpTx);
impl_storable_minicbor!(IcpToEvmTx);
impl_storable_minicbor!(IcpToken);
impl_storable_minicbor!(EvmToken);
impl_storable_minicbor!(BridgePair);
impl_storable_minicbor!(Erc20TwinLedgerSuiteRequest);
impl_storable_minicbor!(DexAction);
impl_storable_minicbor!(SwapType);
impl_storable_minicbor!(UserDexActions);
impl_storable_minicbor!(DexInfo);
