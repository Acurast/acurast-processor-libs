use std::fmt;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StorageQueryResult {
    pub block: String,
    pub changes: Vec<Vec<String>>,
}

impl StorageQueryResult {
    pub fn new(block: String, changes: Vec<Vec<String>>) -> Self {
        StorageQueryResult { block, changes }
    }
}

impl fmt::Display for StorageQueryResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StorageQueryResult(block: {})", self.block)
    }
}