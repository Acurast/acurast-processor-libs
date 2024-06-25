use std::sync::Arc;

use super::v0_acurast_storage::V0AcurastStorage;

#[derive(uniffi::Object)]
pub struct AcurastStorage {
    pub storage: Arc<dyn V0AcurastStorage>,
}

#[uniffi::export]
impl AcurastStorage {
    #[uniffi::constructor]
    pub fn new(storage: Arc<dyn V0AcurastStorage>) -> Arc<Self> {
        Arc::new(Self { storage: storage })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl AcurastStorage {
    async fn get_manager_identifier(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<u32> {
        self.storage.get_manager_identifier(account_id, block_hash, timeout).await
    }
}