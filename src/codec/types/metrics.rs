use num_bigint::BigUint;
use num_traits::Pow;

use crate::cursor::{read_bytes, read_list, read_u128, read_u64};
use std::error::Error;
use std::io::{Cursor, Read};

#[derive(uniffi::Record, Debug)]
pub struct MetricPool {
    pub config: Vec<Config>,
    pub name: String,
    pub reward: Reward,
}

impl MetricPool {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let configs = read_list(cursor, Config::read)?;
        let name = read_bytes(cursor, 24)?;
        let name = String::from_utf8(name).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        let reward = Reward::read(cursor)?;
        let metric = MetricPool { config: configs, name, reward };
        Ok(metric)
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
    pub current: f64,
}

impl Reward {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let current = read_u64(cursor).map(|x| x as f64 / 10f64.pow(18))?;
        Ok(Reward { current })
    }
}
