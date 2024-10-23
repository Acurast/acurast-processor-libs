use super::v0_acurast_storage::VXAcurastStorage;
use crate::{
    codec::types::{
        acurast::{job_identifier::JobIdentifier, job_registration::JobRegistration},
        marketplace::job_assigment::JobAssignment,
        uniques::pallet_uniques_item_details::PalletUniquesItemDetails,
    },
    types::system::FrameSystemAccountInfo,
};
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct AcurastStorage {
    pub storage: Arc<dyn VXAcurastStorage>,
}

#[uniffi::export]
impl AcurastStorage {
    #[uniffi::constructor]
    pub fn new(storage: Arc<dyn VXAcurastStorage>) -> Arc<Self> {
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
        self.storage
            .get_manager_identifier(account_id, block_hash, timeout)
            .await
    }

    async fn get_manager_counter(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<u64> {
        self.storage
            .get_manager_counter(account_id, block_hash, timeout)
            .await
    }

    async fn get_asset(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<PalletUniquesItemDetails> {
        self.storage
            .get_asset(manager_id, block_hash, timeout)
            .await
    }

    async fn get_account_info(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> FrameSystemAccountInfo {
        self.storage
            .get_account_info(account_id, block_hash, timeout)
            .await
    }

    async fn get_assigned_jobs(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Vec<JobAssignment> {
        self.storage
            .get_assigned_jobs(account_id, block_hash, timeout)
            .await
    }

    async fn get_job_registration(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobRegistration> {
        self.storage
            .get_job_registration(job_identifier, block_hash, timeout)
            .await
    }
}
