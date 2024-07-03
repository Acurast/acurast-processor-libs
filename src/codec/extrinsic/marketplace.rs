use std::io::{Cursor, Read};
use byteorder::{ReadBytesExt};
use num_bigint::{BigInt, BigUint};
use std::error::Error;
use std::fmt;
use hex::FromHex;

use crate::big_uint::BigIntToInt;
use crate::cursor::read_compact_integer;

#[derive(uniffi::Enum)]
#[derive(Debug)]
pub enum Tag {
    Secp256r1 = 0,
    Secp256k1 = 1,
    Ed25519 = 2,
}

impl Tag {
    pub fn to_u8a(self) -> u8 {
        self as u8
    }

    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let mut buffer = [0u8; 1];
        cursor.read_exact(&mut buffer)?;
        let value = buffer[0];
        Tag::try_from(value).map_err(|_| format!("Unknown PublicKey tag ({})", value).into())
    }
}

impl TryFrom<u8> for Tag {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        println!("Tag::try_from {}", value);
        match value {
            0 => Ok(Tag::Secp256r1),
            1 => Ok(Tag::Secp256k1),
            2 => Ok(Tag::Ed25519),
            _ => Err(()),
        }
    }
}

#[derive(uniffi::Enum)]
#[derive(Debug)]
pub enum PublicKey {
    Secp256r1(Vec<u8>),
    Secp256k1(Vec<u8>),
    Ed25519(Vec<u8>),
}

impl PublicKey {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let tag = Tag::read(cursor)?;
        let length = read_compact_integer(cursor)?.to_u32();
        let mut value = vec![0; length as usize];
        cursor.read_exact(&mut value)?;

        match tag {
            Tag::Secp256r1 => Ok(PublicKey::Secp256r1(value)),
            Tag::Secp256k1 => Ok(PublicKey::Secp256k1(value)),
            Tag::Ed25519 => Ok(PublicKey::Ed25519(value)),
        }
    }

    fn to_u8a(&self) -> Vec<u8> {
        match self {
            PublicKey::Secp256r1(bytes) => {
                let mut vec = vec![Tag::Secp256r1.to_u8a()];
                vec.extend_from_slice(&bytes);
                vec
            }
            PublicKey::Secp256k1(bytes) => {
                let mut vec = vec![Tag::Secp256k1.to_u8a()];
                vec.extend_from_slice(&bytes);
                vec
            }
            PublicKey::Ed25519(bytes) => {
                let mut vec = vec![Tag::Ed25519.to_u8a()];
                vec.extend_from_slice(&bytes);
                vec
            }
        }
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PublicKey::Secp256r1(bytes) => write!(f, "Secp256r1({:?})", bytes),
            PublicKey::Secp256k1(bytes) => write!(f, "Secp256k1({:?})", bytes),
            PublicKey::Ed25519(bytes) => write!(f, "Ed25519({:?})", bytes),
        }
    }
}