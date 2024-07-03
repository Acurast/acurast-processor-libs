use std::{error::Error, io::{Cursor, Read}};

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

#[derive(uniffi::Record)]
#[derive(Debug)]
pub struct AccountId32 {
    pub bytes: Vec<u8>,
}

impl AccountId32 {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let mut bytes = vec![0; 32];
        cursor.read_exact(&mut bytes)?;
        Ok(AccountId32 { bytes })
    }

    pub fn to_u8a(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}