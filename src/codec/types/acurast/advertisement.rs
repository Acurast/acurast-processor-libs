use std::{error::Error, io::Cursor};

use byteorder::{LittleEndian, ReadBytesExt};
use num_bigint::BigUint;

use crate::{codec::types::acurast::{job_identifier::MultiOrigin, job_registration::JobModule}, cursor::{read_list, read_u128, CursorExt}};

#[derive(uniffi::Record, Debug)]
pub struct MarketplacePricing {
    pub fee_per_millisecond: BigUint,
    pub fee_per_storage_byte: BigUint,
    pub base_fee_per_execution: BigUint,
    pub scheduling_window: SchedulingWindow,
}

impl MarketplacePricing {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let fee_per_millisecond = read_u128(cursor)?;
        let fee_per_storage_byte = read_u128(cursor)?;
        let base_fee_per_execution = read_u128(cursor)?;
        let scheduling_window = SchedulingWindow::read(cursor)?;

        Ok(MarketplacePricing {
            fee_per_millisecond,
            fee_per_storage_byte,
            base_fee_per_execution,
            scheduling_window,
        })
    }
}
#[derive(uniffi::Record, Debug)]
pub struct MarketplaceAdvertisementRestriction {
    pub max_memory: u32,
    pub network_request_quota: u8,
    pub storage_capacity: u32,
    pub allowed_consumers: Option<Vec<MultiOrigin>>,
    pub available_modules: Vec<JobModule>,
}

impl MarketplaceAdvertisementRestriction {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let max_memory = cursor.read_u32::<LittleEndian>()?;
        let network_request_quota = cursor.read_u8()?;
        let storage_capacity = cursor.read_u32::<LittleEndian>()?;
        let allowed_consumers = cursor.read_optional(|c| read_list(c, MultiOrigin::read))?;
        let available_modules = read_list(cursor, JobModule::read)?;

        Ok(MarketplaceAdvertisementRestriction {
            max_memory,
            network_request_quota,
            storage_capacity,
            allowed_consumers,
            available_modules,
        })
    }
}

#[derive(uniffi::Enum, Debug)]
pub enum SchedulingWindow {
    End,
    Delta,
}

impl SchedulingWindow {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let id = cursor.read_u8()?;
        match id {
            0 => Ok(SchedulingWindow::End),
            1 => Ok(SchedulingWindow::Delta),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown SchedulingWindow {}", id),
            ))),
        }
    }
}