use xxhash_rust::xxh64::Xxh64;

pub fn xxh128_unsigned(input: &[u8]) -> [u8; 16] {
    let mut hasher = Xxh64::new(0);
    hasher.update(input);
    let hash1 = hasher.digest();

    hasher.reset(0);
    hasher.update(&input[1..]);
    let hash2 = hasher.digest();

    let mut result = [0u8; 16];
    result[..8].copy_from_slice(&hash1.to_le_bytes());
    result[8..].copy_from_slice(&hash2.to_le_bytes());
    result
}

pub fn xxh128_signed(input: &[u8]) -> [i8; 16] {
    let mut hasher = Xxh64::new(0);
    hasher.update(input);
    let hash1 = hasher.digest();

    hasher.reset(1);
    hasher.update(&input[1..]);
    let hash2 = hasher.digest();

    let mut result = [0i8; 16];
    let hash1_bytes = hash1.to_le_bytes();
    let hash2_bytes = hash2.to_le_bytes();

    for i in 0..8 {
        result[i] = hash1_bytes[i] as i8;
        result[i + 8] = hash2_bytes[i] as i8;
    }

    result
}