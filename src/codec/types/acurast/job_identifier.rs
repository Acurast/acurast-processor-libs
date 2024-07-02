use std::{error::Error, fmt, io::{Cursor, Read}};
use byteorder::ReadBytesExt;
use num_bigint::BigUint;

use crate::{big_uint_utils::read_u128, codec::types::account_id::AccountId32};

#[derive(uniffi::Enum)]
#[derive(Debug)]
enum Kind {
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

#[derive(uniffi::Record)]
#[derive(Debug)]
struct MultiOrigin {
    kind: Kind,
    source: Vec<u8>,
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

    fn to_u8a(&self) -> Vec<u8> {
        let mut bytes = vec![self.kind.to_u8()];
        bytes.extend_from_slice(&self.source);
        bytes
    }
}

impl fmt::Display for MultiOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(kind: {:?}, source: {:?})", self.kind, self.source)
    }
}

#[derive(uniffi::Record)]
#[derive(Debug)]
pub struct JobIdentifier {
    origin: MultiOrigin,
    id: BigUint,
}

impl JobIdentifier {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let origin = MultiOrigin::read(cursor)?;
        let id = read_u128(cursor)?;
        Ok(JobIdentifier { origin, id })
    }
}