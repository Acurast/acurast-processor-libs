use std::io::{Cursor, Read};

#[derive(uniffi::Enum)]
#[derive(Debug)]
pub enum AccountIdentifier {
    AccountID = 0,
    AccountIndex = 1,
    Raw = 2,
    Address32 = 3,
    Address20 = 4,
}

#[derive(uniffi::Record)]
#[derive(Debug)]
pub struct MultiAddress {
    pub account_identifier: AccountIdentifier,
    pub bytes: Vec<u8>
}

impl MultiAddress {
    pub fn read_account_id(buffer: &[u8]) -> Self {
        let mut cursor = Cursor::new(buffer);

        // Ensure the buffer is in little-endian order and read 32 bytes
        let mut ba = vec![0u8; 32];
        cursor.read_exact(&mut ba).unwrap();

        MultiAddress {
            account_identifier: AccountIdentifier::AccountID,
            bytes: ba,
        }
    }
}