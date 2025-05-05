use byteorder::{LittleEndian, ReadBytesExt};
use num_bigint::{BigInt, BigUint};
use num_traits::FromPrimitive;
use std::{
    error::Error,
    io::{Cursor, Read},
};

use crate::{big_uint::BigIntToInt, hex::{hex_to_ba, is_hex}};

pub fn read_changes_opt<T, F>(changes: &Vec<Vec<Option<String>>>, index: usize, read: F) -> Option<T>
where
    F: FnOnce(Vec<u8>) -> T 
{
    let Some(change) = changes.get(index) else {
        return None;
    };

    let Some(Some(value)) = change.get(1) else {
        return None;
    };

    if !is_hex(value) {
        return None;
    }

    let bytes = hex_to_ba(value);

    Some(read(bytes))
}

pub fn read_changes_opt_with_cursor<T, F>(changes: &Vec<Vec<Option<String>>>, index: usize, read: F) -> Option<T>
where
    F: FnOnce(&mut Cursor<Vec<u8>>) -> T 
{
    read_changes_opt(changes, index, |bytes| {
        let mut cursor = Cursor::new(bytes);
        read(&mut cursor)
    })
}

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

pub fn read_optional_u128(cursor: &mut Cursor<Vec<u8>>) -> Result<Option<BigUint>, Box<dyn Error>> {
    let mut flag = [0u8; 1];
    cursor.read_exact(&mut flag)?;

    match flag[0] {
        0 => Ok(None),                     // 0 indicates `None`
        1 => Ok(Some(read_u128(cursor)?)), // 1 indicates `Some`, read the value
        _ => Err(format!("Invalid flag for optional BigUint: {:?}", flag[0]).into()), // Invalid flag
    }
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

pub fn read_list<T, F>(
    cursor: &mut Cursor<Vec<u8>>,
    element_parser: F,
) -> Result<Vec<T>, Box<dyn Error>>
where
    F: Fn(&mut Cursor<Vec<u8>>) -> Result<T, Box<dyn Error>>,
{
    let size = read_compact_integer(cursor)?.to_u32();
    log::debug!("read_list:: {}", size);
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
