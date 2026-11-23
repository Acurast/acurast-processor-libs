use std::mem::transmute;

use num_bigint::BigUint;

// Extension trait for converting integers to byte arrays
pub trait ToU8a {
    fn to_u8a(&self) -> [u8; 4];
}

// Implement the trait for i32
impl ToU8a for i32 {
    fn to_u8a(&self) -> [u8; 4] {
        unsafe { transmute::<i32, [u8; 4]>(*self) }
    }
}

// Implement the trait for u32
impl ToU8a for u32 {
    fn to_u8a(&self) -> [u8; 4] {
        unsafe { transmute::<u32, [u8; 4]>(*self) }
    }
}

pub fn biguint_to_u8a(value: &BigUint) -> [u8; 16] {
    let encoded = value.to_bytes_be();
    let mut result = [0u8; 16];

    let start = 16 - encoded.len();
    result[start..].copy_from_slice(&encoded);
    result.reverse();

    result
}

pub fn biguint_to_u8a_8(value: &BigUint) -> [u8; 8] {
    let encoded = value.to_bytes_be();
    let mut result = [0u8; 8];

    let start = 8 - encoded.len();
    result[start..].copy_from_slice(&encoded);
    result.reverse();

    result
}

pub trait ToSigned {
    fn to_i8(&self) -> Vec<i8>;
}

impl ToSigned for Vec<u8> {
    fn to_i8(&self) -> Vec<i8> {
        self.iter().map(|&x| x as i8).collect()
    }
}

pub trait VecU8aToU8a {
    fn to_u8a(&self) -> Vec<u8>;
}

impl VecU8aToU8a for Vec<u8> {
    fn to_u8a(&self) -> Vec<u8> {
        let size_bytes = self.len().to_compact_u8a();
        [size_bytes, self.to_vec()].concat()
    }
}

enum CompactMode {
    One,
    Two,
    Four,
    Big,
}

impl CompactMode {
    fn value(&self) -> u8 {
        match self {
            CompactMode::One => 0,
            CompactMode::Two => 1,
            CompactMode::Four => 2,
            CompactMode::Big => 3,
        }
    }
}

trait ToCompactU8a {
    fn to_compact_u8a(&self) -> Vec<u8>;
}

impl ToCompactU8a for usize {
    fn to_compact_u8a(&self) -> Vec<u8> {
        let mode = if *self <= 0x3f {
            CompactMode::One
        } else if *self <= 0x3fff {
            CompactMode::Two
        } else if *self <= 0x3fffffff {
            CompactMode::Four
        } else {
            CompactMode::Big
        };

        let mut compact = (*self << 2) + mode.value() as usize;

        let n_bytes = match mode {
            CompactMode::One => 1,
            CompactMode::Two => 2,
            CompactMode::Four => 4,
            CompactMode::Big => {
                panic!("Big mode not implemented");
            }
        };

        let mut ret = Vec::with_capacity(n_bytes);
        for _ in 0..n_bytes {
            ret.push((compact & 0xff) as u8);
            compact >>= 8;
        }

        ret
    }
}
