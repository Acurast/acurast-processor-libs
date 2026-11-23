use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;

/// Hashes the input data using BLAKE2b and returns a 128-bit hash as u128.
pub fn blake2b_128(input: &[u8], hash_size: usize) -> [u8; 16] {
    let mut hasher = Blake2bVar::new(hash_size).expect("Invalid hash size");
    hasher.update(input);
    let mut hash = vec![0u8; hash_size];
    hasher
        .finalize_variable(&mut hash)
        .expect("Failed to finalize hash");

    // Convert to [i8; 16]
    /*let mut signed_hash = [0i8; 16];
    for (i, &byte) in hash.iter().enumerate() {
        signed_hash[i] = byte as i8;
    }*/
    let mut array = [0u8; 16];
    array.copy_from_slice(&hash);
    array
}