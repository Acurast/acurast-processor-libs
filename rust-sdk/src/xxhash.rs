use xxhash_rust::xxh64::Xxh64;

macro_rules! xxh {
    ($name:ident, $segments:literal, $typ:ident) => {
        pub fn $name(input: &[u8]) -> [$typ; $segments * 8] {
            let mut result: [$typ; $segments * 8] = [0; $segments * 8];
            for i in 0..$segments {
                let mut hasher = Xxh64::new(i as u64);
                hasher.update(input);
                let hash = hasher.digest();
                let hash_bytes = hash.to_le_bytes();
                for j in 0..8 {
                    result[i * 8 + j] = hash_bytes[j] as $typ;
                }
            }
            result
        }
    };
}

xxh!(xxh64_unsigned, 1, u8);
xxh!(xxh128_unsigned, 2, u8);
