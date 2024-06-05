use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug, PartialEq)]
pub enum Platform {
    Android, Ios
}

impl Platform {
    pub fn read(value: &mut Cursor<&[u8]>) -> Self {
        let platform_value = value.read_u32::<BigEndian>().expect("Failed to read platform value");
        match platform_value {
            0 => Platform::Android,
            _ => unreachable!("Unknown platform value"),
        }
    }
}