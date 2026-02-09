use std::{error::Error, io::Cursor};

use num_bigint::BigUint;

use crate::cursor::{read_u128, read_u256};

use super::stake::Stake;

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct Delegation {
    pub stake: Stake,
    pub reward_weight: BigUint,
    pub slash_weight: BigUint,
    pub reward_debt: BigUint,
    pub slash_debt: BigUint,
}

impl Delegation {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let stake = Stake::read(cursor)?;
        let reward_weight = read_u256(cursor)?;
        let slash_weight = read_u256(cursor)?;
        let reward_debt = read_u128(cursor)?;
        let slash_debt = read_u128(cursor)?;

        Ok(Self {
            stake,
            reward_weight,
            slash_weight,
            reward_debt,
            slash_debt,
        })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::hex::hex_to_ba;

    use super::*;

    #[test]
    fn test_decode_delegation() {
        let encoded = "0x0090c3dc5348010000000000000000000090c3dc534801000000000000000000df438800807000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000090c3dc534801000000000000000000000000000000000000000000000000000090c3dc53480100000000000000000000000000000000000000000000000000dd2c1e7c6e010000000000000000000000000000000000000000000000000000";
        let encoded = hex_to_ba(encoded);
        let mut cursor = Cursor::new(encoded);
        let delegation = Delegation::read(&mut cursor).unwrap();

        assert_eq!(
            Delegation {
                stake: Stake {
                    amount: BigUint::from_str("361000000000000").unwrap(),
                    rewardable_amount: BigUint::from_str("361000000000000").unwrap(),
                    created: 8_930_271,
                    cooldown_period: 28_800,
                    cooldown_started: None,
                    accured_reward: BigUint::from_str("0").unwrap(),
                    accrued_slash: BigUint::from_str("0").unwrap(),
                    allow_auto_compound: true,
                    paid: BigUint::from_str("0").unwrap(),
                    applied_slash: BigUint::from_str("0").unwrap(),
                },
                reward_weight: BigUint::from_str("361000000000000").unwrap(),
                slash_weight: BigUint::from_str("361000000000000").unwrap(),
                reward_debt: BigUint::from_str("1574040382685").unwrap(),
                slash_debt: BigUint::from_str("0").unwrap(),
            },
            delegation,
        );
    }
}
