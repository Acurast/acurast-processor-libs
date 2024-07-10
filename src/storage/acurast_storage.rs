use std::sync::Arc;
use crate::{codec::types::{marketplace::job_assigment::JobAssignment, uniques::pallet_uniques_item_details::PalletUniquesItemDetails}, types::system::FrameSystemAccountInfo};
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
    ) -> Option<i32> {
        self.storage.get_manager_identifier(account_id, block_hash, timeout).await
    }

    async fn get_asset(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<PalletUniquesItemDetails> {
        self.storage.get_asset(manager_id, block_hash, timeout).await
    }

    async fn get_account_info(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> FrameSystemAccountInfo {
        self.storage.get_account_info(account_id, block_hash, timeout).await
    }

    async fn get_assigned_jobs(
        &self, 
        account_id: &[u8], 
        block_hash: Option<Vec<u8>>, 
        timeout: Option<u64>
    ) -> Vec<JobAssignment> {
        self.storage.get_assigned_jobs(account_id, block_hash, timeout).await
    }
}