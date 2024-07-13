use std::{error::Error, fmt, io::{Cursor, Read}};
use byteorder::ReadBytesExt;
use num_bigint::BigUint;

use crate::{cursor::read_u128, codec::types::account_id::AccountId32, u8a::ToSigned};

#[derive(uniffi::Enum)]
#[derive(Debug)]
pub enum Kind {
    Acurast,
    Tezos,
}

impl Kind {
    fn from_byte(byte: u8) -> Result<Self, Box<dyn Error>> {
        match byte {
            0 => Ok(Kind::Acurast),
            1 => Ok(Kind::Tezos),
            _ => Err(format!("Unknown multi origin kind {}", byte).into()),
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            Kind::Acurast => 0,
            Kind::Tezos => 1,
        }
    }
}

impl Clone for Kind {
    fn clone(&self) -> Self {
        Kind::from_byte(self.to_u8()).unwrap()
    }
}

#[derive(uniffi::Record)]
#[derive(Debug)]
pub struct MultiOrigin {
    pub kind: Kind,
    pub source: Vec<u8>,
}

impl MultiOrigin {
    fn acurast(address: AccountId32) -> Self {
        MultiOrigin {
            kind: Kind::Acurast,
            source: address.to_u8a(),
        }
    }

    fn tezos(payload: Vec<u8>) -> Self {
        MultiOrigin {
            kind: Kind::Tezos,
            source: payload,
        }
    }

    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let kind_byte = cursor.read_u8()?;
        let kind = Kind::from_byte(kind_byte)?;

        let source = match kind {
            Kind::Acurast => {
                let address = AccountId32::read(cursor)?;
                address.to_u8a()
            },
            Kind::Tezos => {
                let payload_length = cursor.read_u8()? as usize;
                let mut payload = vec![0; payload_length];
                cursor.read_exact(&mut payload)?;
                payload
            }
        };

        Ok(MultiOrigin { kind, source })
    }

    pub fn to_u8a(&self) -> Vec<u8> {
        let mut bytes = vec![self.kind.to_u8()];
        bytes.extend_from_slice(&self.source);
        bytes
    }
}

impl fmt::Display for MultiOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(kind: {:?}, source: {:?})", self.kind, self.source.to_i8())
    }
}

impl Clone for MultiOrigin {
    fn clone(&self) -> Self {
        MultiOrigin { kind: self.kind.clone(), source: self.source.clone() }
    }
}

#[derive(uniffi::Record)]
#[derive(Debug)]
pub struct JobIdentifier {
    pub origin: MultiOrigin,
    pub id: BigUint,
}

impl JobIdentifier {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let origin = MultiOrigin::read(cursor)?;
        let id = read_u128(cursor)?;
        Ok(JobIdentifier { origin, id })
    }

    pub fn empty() -> Self {
        JobIdentifier { origin: MultiOrigin { kind: Kind::Acurast, source: vec![] }, id: BigUint::ZERO }
    }
}

impl fmt::Display for JobIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "job_identifier = (origin: {:?}, id: {:?})", self.origin, self.id.to_str_radix(10))
    }
}

impl Clone for JobIdentifier {
    fn clone(&self) -> Self {
        JobIdentifier { origin: self.origin.clone(), id: self.id.clone() }
    }
}