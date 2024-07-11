use std::{error::Error, io::{Cursor, Read}};
use byteorder::{LittleEndian, ReadBytesExt};
use num_bigint::{BigInt, BigUint};
use num_traits::FromPrimitive;

use crate::big_uint::BigIntToInt;

pub fn read_bytes(cursor: &mut Cursor<Vec<u8>>, offset: usize) -> Result<Vec<i8>, Box<dyn Error>> {
    let mut ba = vec![0u8; offset];
    cursor.read_exact(&mut ba)?;

    // Convert Vec<u8> to Vec<i8>
    let ba_signed: Vec<i8> = ba.into_iter().map(|x| x as i8).collect();
    
    Ok(ba_signed)
}

pub fn read_u128(cursor: &mut Cursor<Vec<u8>>) -> Result<BigUint, Box<dyn Error>> {
    let mut bytes = [0u8; 16];
    cursor.read_exact(&mut bytes)?;
    Ok(BigUint::from_bytes_le(&bytes))
}

pub fn read_compact_integer(cursor: &mut Cursor<Vec<u8>>) -> Result<BigInt, Box<dyn Error>> {
    let compact_byte = cursor.read_u8()? as u64;
    
    let compact_length = match compact_byte % 4 {
        0 => 1,
        1 => 2,
        2 => 4,
        _ => 5 + (compact_byte - 3) / 4,
    };
    let result = match compact_length {
        1 => BigInt::from_u64(compact_byte >> 2).unwrap(),
        2 => {
            cursor.set_position(cursor.position() - 1);
            let short = cursor.read_u16::<LittleEndian>()? as u64;
            BigInt::from_u64(short >> 2).unwrap()
        }
        4 => {
            let mut ba = vec![0u8; compact_length as usize - 1];
            cursor.read_exact(&mut ba)?;
            let ba = [&[compact_byte as u8][..], &ba[..]].concat();
            BigInt::from_bytes_be(num_bigint::Sign::Plus, &ba) >> 2
        }
        _ => {
            let mut ba = vec![0u8; compact_length as usize - 1];
            cursor.read_exact(&mut ba)?;
            BigInt::from_bytes_be(num_bigint::Sign::Plus, &ba)
        }
    };

    Ok(result)
}

pub fn read_list<T, F>(cursor: &mut Cursor<Vec<u8>>, element_parser: F) -> Result<Vec<T>, Box<dyn Error>>
where
    F: Fn(&mut Cursor<Vec<u8>>) -> Result<T, Box<dyn Error>>,
{
    let size = read_compact_integer(cursor)?.to_u32();
    println!("read_list:: {}", size);
    let mut list = Vec::with_capacity(size as usize);

    for _ in 0..size {
        list.push(element_parser(cursor)?);
    }

    Ok(list)
}

pub fn read_byte_array(cursor: &mut Cursor<Vec<u8>>) -> Result<Vec<u8>, Box<dyn Error>> {
    let length = read_compact_integer(cursor)?.to_u32() as usize;
    let mut buffer = vec![0; length];
    cursor.read_exact(&mut buffer)?;
    Ok(buffer)
}

pub trait CursorExt {
    fn read_boolean(&mut self) -> Result<bool, Box<dyn Error>>;
    fn read_optional_boolean(&mut self) -> Result<Option<bool>, Box<dyn Error>>;
    fn read_optional<T, F>(&mut self, optional_parser: F) -> Result<Option<T>, Box<dyn Error>>
    where
        F: Fn(&mut Cursor<Vec<u8>>) -> Result<T, Box<dyn Error>>;
}

impl CursorExt for Cursor<Vec<u8>> {
    fn read_boolean(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut byte = [0u8; 1];
        self.read_exact(&mut byte)?;
        Ok(byte[0] == 0x01)
    }

    fn read_optional_boolean(&mut self) -> Result<Option<bool>, Box<dyn Error>> {
        match self.read_boolean()? {
            true => Ok(Some(true)),
            false => Ok(Some(false)),
        }
    }

    fn read_optional<T, F>(&mut self, optional_parser: F) -> Result<Option<T>, Box<dyn Error>>
    where
        F: Fn(&mut Cursor<Vec<u8>>) -> Result<T, Box<dyn Error>>,
    {
        let is_present = self.read_boolean()?;
        if is_present {
            optional_parser(self).map(Some)
        } else {
            Ok(None)
        }
    }
}