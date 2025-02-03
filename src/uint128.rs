#[derive(uniffi::Record, Debug, Clone)]
pub struct UInt128 {
    pub bytes: Vec<u8>, // Must always be 16 bytes
}

impl UInt128 {
    pub fn new(value: u128) -> Self {
        Self {
            bytes: value.to_le_bytes().to_vec(), // Convert u128 to 16 bytes
        }
    }

    pub fn to_u128(&self) -> u128 {
        let mut array = [0u8; 16];
        array.copy_from_slice(&self.bytes); // Convert Vec<u8> back to u128
        u128::from_le_bytes(array)
    }
}