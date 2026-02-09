use std::{error::Error, io::Cursor};

use num_bigint::BigUint;

use crate::cursor::{read_u128, read_u32};


#[derive(uniffi::Record, Debug, PartialEq)]
pub struct TokenConversion {
    pub amount: BigUint,
    pub lock_start: u32,
}

impl TokenConversion {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let amount = read_u128(cursor)?;
        let lock_start = read_u32(cursor)?;

        Ok(Self { amount, lock_start })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::hex::hex_to_ba;

    use super::*;

    #[test]
    fn test_decode_token_conversion() {
        let encoded = "0x987367e90aa60400000000000000000098451200";
        let encoded = hex_to_ba(encoded);
        let mut cursor = Cursor::new(encoded);
        let token_conversion = TokenConversion::read(&mut cursor).unwrap();

        assert_eq!(
            TokenConversion {
                amount: BigUint::from_str("1308465702597528").unwrap(),
                lock_start: 1_197_464,
            },
            token_conversion,
        );
    }
}
