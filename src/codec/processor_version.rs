use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use crate::codec::platform::Platform;

#[derive(uniffi::Object)] 
#[derive(Debug)]
pub struct ProcessorVersion {
    pub platform: Platform,
    pub build_number: u32,
}

#[uniffi::export]
impl ProcessorVersion {
    pub fn to_u8a(&self) -> Vec<u8> {
        let mut result = self.platform.to_u8a();
        result.extend_from_slice(&self.build_number.to_le_bytes());
        result
    }
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