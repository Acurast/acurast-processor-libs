use super::v0_acurast_storage::VXAcurastStorage;
use crate::codec::types::account_overview::AccountOverview;
use crate::Result;
use crate::xxhash::{xxh128_unsigned, xxh64_unsigned};
use crate::blake2b::blake2b_128;
use crate::codec::types::account_info::AccountInfo;
use crate::codec::types::acurast::job_registration::JobRegistration;
use crate::codec::types::acurast::job_identifier::JobIdentifier;
use crate::codec::types::acurast::job_environment::JobEnvironment;
use crate::codec::types::acurast::attestation::Attestation;
use crate::codec::types::acurast::advertisement::{MarketplaceAdvertisementRestriction, MarketplacePricing};
use crate::codec::types::uniques::pallet_uniques_item_details::PalletUniquesItemDetails;
use crate::codec::types::processor_overview::ProcessorOverview;
use crate::codec::types::marketplace::job_assigment::JobAssignment;
use crate::codec::types::management_data::ManagementData;
use crate::codec::types::job_data::JobData;
use crate::codec::types::compute::metrics::MetricPool;
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct AcurastStorage {
    pub storage: Arc<dyn VXAcurastStorage>,
}

#[uniffi::export]
impl AcurastStorage {
    #[uniffi::constructor]
    pub fn new(storage: Arc<dyn VXAcurastStorage>) -> Arc<Self> {
        Arc::new(Self { storage })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl AcurastStorage {
    async fn get_manager_identifier(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<i32>> {
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
    ) -> AccountInfo {
        self.storage
            .get_account_info(account_id, block_hash, timeout)
            .await
    }

    async fn get_account_overview(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<AccountOverview>> {
        self.storage
            .get_account_overview(account_id, block_hash, timeout)
            .await
    }

    async fn get_assigned_jobs(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<JobAssignment>> {
        self.storage
            .get_assigned_jobs(account_id, block_hash, timeout)
            .await
    }

    async fn get_job_registration(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobRegistration>> {
        self.storage
            .get_job_registration(job_identifier, block_hash, timeout)
            .await
    }

    async fn get_job_data(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobData>> {
        self.storage
            .get_job_data(job_identifier, block_hash, timeout)
            .await
    }

    async fn get_job_environment(
        &self,
        job_identifier: JobIdentifier,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobEnvironment>> {
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
    ) -> Result<Option<Attestation>> {
        self.storage
            .get_attestation(account_id, block_hash, timeout)
            .await
    }

    async fn get_marketplace_pricing(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<MarketplacePricing>> {
        self.storage
            .get_marketplace_pricing(account_id, block_hash, timeout)
            .await
    }

    async fn get_marketplace_advertisement_restriction(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<MarketplaceAdvertisementRestriction>> {
        self.storage
            .get_marketplace_advertisement_restriction(account_id, block_hash, timeout)
            .await
    }

    async fn get_metric_pool_ids(
        &self,
        names: Vec<String>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<Option<u8>>> {
        self.storage
            .get_metric_pool_ids(names, block_hash, timeout)
            .await
    }

    async fn get_metric_pools(
        &self,
        ids: Vec<u8>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<Option<MetricPool>>> {
        self.storage
            .get_metric_pools(ids, block_hash, timeout)
            .await
    }

    async fn get_management_data(
        &self,
        manager_id: i32,
        manager_account_id: Option<Vec<u8>>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<ManagementData>> {
        self.storage
            .get_management_data(manager_id, manager_account_id, block_hash, timeout)
            .await
    }

    async fn get_processor_overview(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<ProcessorOverview> {
        self.storage
            .get_processor_overview(account_id, block_hash, timeout)
            .await
    }
}

pub trait StorageKey {
    fn extend_with_storage_prefix(&mut self, module: &str, method: &str);
    fn extend_with_blake2_128(&mut self, bytes: &[u8]);
    fn extend_with_blake2_128concat(&mut self, bytes: &[u8]);
    fn extend_with_xxh64(&mut self, bytes: &[u8]);
    fn extend_with_xxh64concat(&mut self, bytes: &[u8]);
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

    fn extend_with_xxh64(&mut self, bytes: &[u8]) {
        self.extend_from_slice(&xxh64_unsigned(bytes));
    }

    fn extend_with_xxh64concat(&mut self, bytes: &[u8]) {
        self.extend_with_xxh64(bytes);
        self.extend_from_slice(bytes);
    }
}
