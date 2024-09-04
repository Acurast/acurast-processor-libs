use xxhash_rust::xxh64::Xxh64;

pub fn xxh128_unsigned(input: &[u8]) -> [u8; 16] {
    let mut hasher1 = Xxh64::new(0);
    hasher1.update(input);
    let hash1 = hasher1.digest();

    let mut hasher2 = Xxh64::new(1);
    hasher2.update(input);
    let hash2 = hasher2.digest();

    let mut result = [0u8; 16];
    let hash1_bytes = hash1.to_le_bytes();
    let hash2_bytes = hash2.to_le_bytes();

    for i in 0..8 {
        result[i] = hash1_bytes[i];
        result[i + 8] = hash2_bytes[i];
    }

    result
}

pub fn xxh128_signed(input: &[u8]) -> [i8; 16] {
    let mut hasher1 = Xxh64::new(0);
    hasher1.update(input);
    let hash1 = hasher1.digest();

    let mut hasher2 = Xxh64::new(1);
    hasher2.update(input);
    let hash2 = hasher2.digest();

    let mut result = [0i8; 16];
    let hash1_bytes = hash1.to_le_bytes();
    let hash2_bytes = hash2.to_le_bytes();

    for i in 0..8 {
        result[i] = hash1_bytes[i] as i8;
        result[i + 8] = hash2_bytes[i] as i8;
    }

    result
}
