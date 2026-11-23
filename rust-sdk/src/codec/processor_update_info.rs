use crate::codec::processor_version::ProcessorVersion;
use std::io::{Cursor, Read};

#[derive(uniffi::Record, Debug)]
pub struct ProcessorUpdateInfo {
    pub version: ProcessorVersion,
    pub binary_location: Vec<u8>,
}

impl ProcessorUpdateInfo {
    pub fn read(value: &mut Cursor<&[u8]>) -> Self {
        let version = ProcessorVersion::read(value);
        let mut binary_location = Vec::new();
        value
            .read_to_end(&mut binary_location)
            .expect("Failed to read binary location");
        Self {
            version,
            binary_location,
        }
    }
}
