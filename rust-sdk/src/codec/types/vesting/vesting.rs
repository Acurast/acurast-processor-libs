use std::{error::Error, io::Cursor};

use num_bigint::BigUint;

use crate::cursor::{read_u128, read_u32};


#[derive(uniffi::Record, Debug, PartialEq)]
pub struct Vesting {
    pub locked: BigUint,
    pub per_block: BigUint,
    pub starting_block: u32,
}

impl Vesting {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let locked = read_u128(cursor)?;
        let per_block = read_u128(cursor)?;
        let starting_block = read_u32(cursor)?;

        Ok(Self { locked, per_block, starting_block })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{cursor::read_list, hex::hex_to_ba};

    use super::*;

    #[test]
    fn test_decode_vesting_list() {
        let encoded = "0x040080d8d59a52300400000000000000009262cd840400000000000000000000002cbc3900";
        let encoded = hex_to_ba(encoded);
        let mut cursor = Cursor::new(encoded);
        let vesting = read_list(&mut cursor, |c| Vesting::read(c)).unwrap();

        assert_eq!(
            vec![
                Vesting {
                    locked: BigUint::from_str("301832000000000000").unwrap(),
                    per_block: BigUint::from_str("19407921810").unwrap(),
                    starting_block: 3_783_724,
                },
            ],
            vesting,
        );
    }
}
