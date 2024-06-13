pub mod engine;
pub mod acurast_rpc;
pub mod storage;
pub mod codec;
pub mod types;
pub mod tests;
pub mod hex;

uniffi::include_scaffolding!("acurastsdk");
use std::str::FromStr;

pub use acurast_rpc::*;
pub use crate::types::header::Header;
use num_bigint::BigUint;


impl UniffiCustomTypeConverter for BigUint {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(BigUint::from_str(&val).unwrap())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}