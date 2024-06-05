use num_bigint::BigUint;

pub struct Header {
    parent_hash: String,
    number: BigUint,
    state_root: String,
    extrinsics_root: String,
}

impl Header {
    pub fn new(parent_hash: String, number: BigUint, state_root: String, extrinsics_root: String) -> Self {
        Self {
            parent_hash,
            number,
            state_root,
            extrinsics_root,
        }
    }

    // Getters
    pub fn parent_hash(&self) -> &str {
        &self.parent_hash
    }

    pub fn number(&self) -> &BigUint {
        &self.number
    }

    pub fn state_root(&self) -> &str {
        &self.state_root
    }

    pub fn extrinsics_root(&self) -> &str {
        &self.extrinsics_root
    }
}