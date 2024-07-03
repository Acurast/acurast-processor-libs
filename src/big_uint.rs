use num_bigint::{BigInt, BigUint};
use std::u32;
use num_traits::Signed;

pub trait BigIntToInt {
    fn to_u32(&self) -> u32;
}

impl BigIntToInt for BigInt {
    fn to_u32(&self) -> u32 {
        let was_negative = self.is_negative();
        let abs = self.abs().to_biguint().unwrap();
        let mut truncated = abs.to_u32();
        if was_negative {
           truncated.wrapping_neg()
        } else {
            truncated
        }
    }
}

impl BigIntToInt for BigUint {
    fn to_u32(&self) -> u32 {
        let mask = BigUint::from(u32::MAX);
        (self & mask).to_u32_digits().first().unwrap().clone()
    }
}