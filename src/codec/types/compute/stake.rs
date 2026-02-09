use std::{error::Error, io::Cursor};

use num_bigint::BigUint;

use crate::cursor::{read_u128, read_u32, CursorExt};

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct Stake {
    pub amount: BigUint,
    pub rewardable_amount: BigUint,
    pub created: u32,
    pub cooldown_period: u32,
    pub cooldown_started: Option<u32>,
    pub accured_reward: BigUint,
    pub accrued_slash: BigUint,
    pub allow_auto_compound: bool,
    pub paid: BigUint,
    pub applied_slash: BigUint,
}

impl Stake {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let amount = read_u128(cursor)?;
        let rewardable_amount = read_u128(cursor)?;
        let created = read_u32(cursor)?;
        let cooldown_period = read_u32(cursor)?;
        let cooldown_started = cursor.read_optional(|c| read_u32(c))?;
        let accured_reward = read_u128(cursor)?;
        let accrued_slash = read_u128(cursor)?;
        let allow_auto_compound = cursor.read_boolean()?;
        let paid = read_u128(cursor)?;
        let applied_slash = read_u128(cursor)?;

        Ok(Self {
            amount,
            rewardable_amount,
            created,
            cooldown_period,
            cooldown_started,
            accured_reward,
            accrued_slash,
            allow_auto_compound,
            paid,
            applied_slash,
        })
    }
}
