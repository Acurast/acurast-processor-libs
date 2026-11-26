use num_bigint::{BigInt, BigUint};

/// Reads SCALE-encoded data, failing with a description on malformed or truncated input.
pub(crate) struct ScaleReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

pub(crate) type ReadResult<T> = Result<T, String>;

impl<'a> ScaleReader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    pub(crate) fn read_bytes(&mut self, count: usize) -> ReadResult<Vec<u8>> {
        if count > self.remaining() {
            return Err("Unexpected end of data".into());
        }
        let bytes = self.bytes[self.offset..self.offset + count].to_vec();
        self.offset += count;
        Ok(bytes)
    }

    pub(crate) fn read_u8(&mut self) -> ReadResult<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    pub(crate) fn read_u32(&mut self) -> ReadResult<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(crate) fn read_bool(&mut self) -> ReadResult<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("Invalid boolean".into()),
        }
    }

    /// Unsigned little-endian integer of `byte_count` bytes.
    pub(crate) fn read_unsigned(&mut self, byte_count: usize) -> ReadResult<BigUint> {
        Ok(BigUint::from_bytes_le(&self.read_bytes(byte_count)?))
    }

    /// Two's complement little-endian integer of `byte_count` bytes.
    pub(crate) fn read_signed(&mut self, byte_count: usize) -> ReadResult<BigInt> {
        Ok(BigInt::from_signed_bytes_le(&self.read_bytes(byte_count)?))
    }

    pub(crate) fn read_compact(&mut self) -> ReadResult<BigUint> {
        let first = self.read_u8()?;
        match first & 0b11 {
            0b00 => Ok(BigUint::from(first >> 2)),
            0b01 => {
                let value = u16::from(first) | u16::from(self.read_u8()?) << 8;
                Ok(BigUint::from(value >> 2))
            }
            0b10 => {
                let rest = self.read_bytes(3)?;
                let value = u32::from_le_bytes([first, rest[0], rest[1], rest[2]]);
                Ok(BigUint::from(value >> 2))
            }
            _ => self.read_unsigned(usize::from(first >> 2) + 4),
        }
    }

    /// A compact length or index, which must fit a `usize`.
    pub(crate) fn read_compact_usize(&mut self) -> ReadResult<usize> {
        let value = self.read_compact()?;
        usize::try_from(&value).map_err(|_| "Compact value too large".to_string())
    }

    pub(crate) fn read_compact_u32(&mut self) -> ReadResult<u32> {
        let value = self.read_compact()?;
        u32::try_from(&value).map_err(|_| "Compact value too large".to_string())
    }

    pub(crate) fn read_byte_array(&mut self) -> ReadResult<Vec<u8>> {
        let count = self.read_compact_usize()?;
        self.read_bytes(count)
    }

    pub(crate) fn read_string(&mut self) -> ReadResult<String> {
        String::from_utf8(self.read_byte_array()?).map_err(|_| "Invalid UTF-8".to_string())
    }

    pub(crate) fn read_optional<T>(
        &mut self,
        read: impl FnOnce(&mut Self) -> ReadResult<T>,
    ) -> ReadResult<Option<T>> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => read(self).map(Some),
            _ => Err("Invalid option".into()),
        }
    }

    pub(crate) fn read_list<T>(
        &mut self,
        mut read: impl FnMut(&mut Self) -> ReadResult<T>,
    ) -> ReadResult<Vec<T>> {
        let count = self.read_compact_usize()?;
        // Every element takes at least one byte here: a longer list can only be malformed.
        if count > self.remaining() {
            return Err("List longer than the remaining data".into());
        }
        (0..count).map(|_| read(self)).collect()
    }
}
