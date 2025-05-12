use num_bigint::BigUint;

use crate::cursor::{read_list, read_u128};
use std::error::Error;
use std::io::{Cursor, Read};

#[derive(uniffi::Record, Debug)]
pub struct MetricPool {
    pub config: Vec<Config>,
}

impl MetricPool {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let configs = read_list(cursor, Config::read)?;
        let metrict = MetricPool { config: configs };
        Ok(metrict)
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

