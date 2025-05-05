use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

fn pure_hex_to_ba(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[uniffi::export]
pub fn hex_to_ba(hex: &str) -> Vec<u8> {
    let hex_str = hex.trim_start_matches("0x");
    pure_hex_to_ba(hex_str)
}

pub fn is_hex(string: &str) -> bool {
    let string = string.strip_prefix("0x").unwrap_or(string);
    string.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn read_u128(bytes: &[u8]) -> u128 {
    let mut arr = [0u8; 16];
    arr.copy_from_slice(bytes);
    let mut cursor = Cursor::new(arr);
    cursor.read_u128::<LittleEndian>().unwrap()
}

pub fn read_u64(bytes: &[u8]) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(bytes);
    let mut cursor = Cursor::new(arr);
    cursor.read_u64::<LittleEndian>().unwrap()
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for Vec<u8> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}
