use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use crate::codec::platform::Platform;

#[derive(Debug)]
pub struct ProcessorVersion {
    platform: Platform,
    build_number: u32,
}

impl ProcessorVersion {
    pub fn read(value: &mut Cursor<&[u8]>) -> Self {
        let platform = Platform::read(value);
        let build_number = value.read_u32::<BigEndian>().expect("Failed to read build number");
        Self {
            platform,
            build_number,
        }
    }
}