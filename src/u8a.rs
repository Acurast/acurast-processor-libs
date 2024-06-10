use std::mem::transmute;

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
