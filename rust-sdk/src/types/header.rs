use num_bigint::BigUint;

#[derive(uniffi::Record)]
pub struct Header {
    pub parent_hash: String,
    pub number: BigUint,
    pub state_root: String,
    pub extrinsics_root: String,
}

impl Header {
    pub fn new(
        parent_hash: String,
        number: BigUint,
        state_root: String,
        extrinsics_root: String,
    ) -> Self {
        Self {
            parent_hash,
            number,
            state_root,
            extrinsics_root,
        }
    }
}
