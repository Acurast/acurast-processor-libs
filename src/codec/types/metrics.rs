use byteorder::ReadBytesExt;
use num_bigint::BigUint;
use num_rational::BigRational;
use num_traits::Pow;

use crate::cursor::{read_bytes, read_fixed_u128, read_list, read_perquintill, read_u128, read_u32, read_u64, CursorExt};
use std::error::Error;
use std::io::{Cursor, Read};

#[derive(uniffi::Record, Debug)]
pub struct MetricPool {
    pub config: Vec<Config>,
    pub name: String,
    pub reward: Reward,
    pub total: Total,
}

impl MetricPool {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let configs = read_list(cursor, Config::read)?;
        let name = read_bytes(cursor, 24)?;
        let name = String::from_utf8(name).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        let reward = Reward::read(cursor)?;
        let total = Total::read(cursor)?;

        Ok(MetricPool { config: configs, name, reward, total })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct Config {
    pub name: String,
    pub numerator: BigUint,
    pub denominator: BigUint,
}

impl Config {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let mut name_bytes = [0u8; 24];
        cursor.read_exact(&mut name_bytes)?;
        let name =
            String::from_utf8(name_bytes.to_vec()).map_err(|e| Box::new(e) as Box<dyn Error>)?;

        let numerator = read_u128(cursor)?;
        let denominator = read_u128(cursor)?;

        Ok(Config {
            name,
            numerator,
            denominator,
        })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct Reward {
    pub current: BigRational,
    pub next: Option<Next>,
}

impl Reward {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let current = read_perquintill(cursor)?;
        let next = cursor.read_optional(Next::read)?;

        Ok(Reward { current, next })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct Next {
    epoch: u32,
    value: BigRational,
}

impl Next {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let epoch = read_u32(cursor)?;
        let value = read_perquintill(cursor)?;

        Ok(Next { epoch, value })
    }
}


#[derive(uniffi::Record, Debug)]
pub struct Total {
    pub epoch: u32,
    pub prev: BigRational,
    pub cur: BigRational,
}

impl Total {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let epoch = read_u32(cursor)?;
        let prev = read_fixed_u128(cursor)?;
        let cur = read_fixed_u128(cursor)?;

        Ok(Total { epoch, prev, cur })
    }
}
