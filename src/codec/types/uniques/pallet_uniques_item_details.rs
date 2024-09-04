use std::io::{Cursor, Read};

use crate::codec::types::account_id::MultiAddress;

#[derive(uniffi::Record, Debug)]
pub struct PalletUniquesItemDetails {
    pub owner: MultiAddress,
}

impl PalletUniquesItemDetails {
    pub fn read(value: &mut Cursor<&[u8]>) -> Self {
        let mut binary_location = Vec::new();
        value
            .read_to_end(&mut binary_location)
            .expect("Failed to read binary location");
        let multiaddres = MultiAddress::read_account_id(binary_location.as_slice());
        Self { owner: multiaddres }
    }
}
