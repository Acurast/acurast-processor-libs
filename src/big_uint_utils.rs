use std::{error::Error, io::{Cursor, Read}};
use num_bigint::BigUint;

pub fn read_u128(cursor: &mut Cursor<Vec<u8>>) -> Result<BigUint, Box<dyn Error>> {
    let mut bytes = [0u8; 16];
    cursor.read_exact(&mut bytes)?;
    Ok(BigUint::from_bytes_be(&bytes))
}