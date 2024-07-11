use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
use std::u32;
use num_traits::Signed;

pub trait BigIntToInt {
    fn to_u32(&self) -> u32;
    fn to_u8a(&self) -> Vec<u8>;
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

    fn to_u8a(&self) -> Vec<u8> {
        let mut encoded = self.to_signed_bytes_be();
        let pad_length = 16 - encoded.len();
        
        let mut result = vec![0; pad_length];
        result.extend(encoded);
        
        result.reverse();
        result
    }
}

impl BigIntToInt for BigUint {
    fn to_u32(&self) -> u32 {
        let mask = BigUint::from(u32::MAX);
        match (self & mask).to_u32_digits().first() {
            Some(&digit) => digit,
            None => 0,
        }
    }

    fn to_u8a(&self) -> Vec<u8> {
        let mut encoded = self.to_bigint().unwrap().to_signed_bytes_le();
        let pad_length = 16 - encoded.len();
        
        let mut result = vec![0; pad_length];
        result.extend(encoded);
        
        result.reverse();
        result
    }
}