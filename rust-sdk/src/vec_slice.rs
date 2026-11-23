pub fn i8_slice_to_u8_slice(slice: &[i8]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) }
}
