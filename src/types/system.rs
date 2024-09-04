use byteorder::{LittleEndian, ReadBytesExt};
use num_bigint::BigUint;
use num_traits::Zero;
use std::io::{Cursor, Read};

#[derive(uniffi::Record, Debug)]
pub struct FrameSystemAccountInfoData {
    pub free: BigUint,
    pub reserved: BigUint,
    pub misc_frozen: BigUint,
    pub fee_frozen: BigUint,
}

#[derive(uniffi::Record, Debug)]
pub struct FrameSystemAccountInfo {
    pub nonce: u32,
    pub consumers: u32,
    pub providers: u32,
    pub sufficients: u32,
    pub data: FrameSystemAccountInfoData,
}

impl FrameSystemAccountInfo {
    pub fn read_account_info(buffer: &[u8]) -> Self {
        let mut cursor = Cursor::new(buffer);

        let nonce = cursor.read_u32::<LittleEndian>().unwrap();
        let consumers = cursor.read_u32::<LittleEndian>().unwrap();
        let providers = cursor.read_u32::<LittleEndian>().unwrap();
        let sufficients = cursor.read_u32::<LittleEndian>().unwrap();

        let data = FrameSystemAccountInfoData {
            free: Self::read_u128(&mut cursor),
            reserved: Self::read_u128(&mut cursor),
            misc_frozen: Self::read_u128(&mut cursor),
            fee_frozen: Self::read_u128(&mut cursor),
        };

        FrameSystemAccountInfo {
            nonce,
            consumers,
            providers,
            sufficients,
            data,
        }
    }

    pub fn empty() -> Self {
        let data = FrameSystemAccountInfoData {
            free: BigUint::zero(),
            reserved: BigUint::zero(),
            misc_frozen: BigUint::zero(),
            fee_frozen: BigUint::zero(),
        };
        FrameSystemAccountInfo {
            nonce: 0,
            consumers: 0,
            providers: 0,
            sufficients: 0,
            data: data,
        }
    }

    fn read_u128(cursor: &mut Cursor<&[u8]>) -> BigUint {
        let mut buf = [0u8; 16];
        cursor.read_exact(&mut buf).unwrap();
        BigUint::from_bytes_le(&buf)
    }
}
