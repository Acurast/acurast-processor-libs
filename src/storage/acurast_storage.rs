use super::v0_acurast_storage::VXAcurastStorage;
use crate::{
    blake2b::blake2b_128, codec::types::{
        acurast::{attestation::Attestation, job_environment::JobEnvironment, job_identifier::JobIdentifier, job_registration::JobRegistration}, marketplace::job_assigment::JobAssignment, metrics::MetricPool, uniques::pallet_uniques_item_details::PalletUniquesItemDetails
    }, types::system::FrameSystemAccountInfo, xxhash::xxh128_unsigned
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

    async fn get_job_environment(
        &self,
        job_identifier: JobIdentifier,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobEnvironment> {
        self.storage
            .get_job_environment(job_identifier, account_id, block_hash, timeout)
            .await
    }

    async fn is_attested(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> bool {
        self.storage
            .is_attested(account_id, block_hash, timeout)
            .await
    }
    
    async fn get_attestation(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Attestation> {
        self.storage
            .get_attestation(account_id, block_hash, timeout)
            .await
    }

    async fn get_metric_pool_ids(
        &self,
        names: Vec<String>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Vec<u8> {
        self.storage
            .get_metric_pool_ids(names, block_hash, timeout)
            .await
    }

    async fn get_metric_pools(
        &self,
        ids: Vec<u8>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Vec<MetricPool> {
        self.storage.get_metric_pools(ids, block_hash, timeout).await
    }

    async fn get_management_endpoint(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Vec<u8>> {
        self.storage.get_management_endpoint(manager_id, block_hash, timeout).await
    }
}

pub trait StorageKey {
    fn extend_with_storage_prefix(&mut self, module: &str, method: &str);
    fn extend_with_blake2_128(&mut self, bytes: &[u8]);
    fn extend_with_blake2_128concat(&mut self, bytes: &[u8]);
}

impl StorageKey for Vec<u8> {
    fn extend_with_storage_prefix(&mut self, module: &str, method: &str) {
        self.extend_from_slice(&xxh128_unsigned(module.as_bytes()));
        self.extend_from_slice(&xxh128_unsigned(method.as_bytes()));
    }

    fn extend_with_blake2_128(&mut self, bytes: &[u8]) {
        self.extend_from_slice(&blake2b_128(bytes, 16));
    }

    fn extend_with_blake2_128concat(&mut self, bytes: &[u8]) {
        self.extend_with_blake2_128(bytes);
        self.extend_from_slice(bytes);
    }
}