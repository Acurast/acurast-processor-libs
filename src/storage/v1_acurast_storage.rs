use super::v0_acurast_storage::{V0AcurastStorageImpl, VXAcurastStorage};
use crate::codec::types::acurast::advertisement::{MarketplaceAdvertisementRestriction, MarketplacePricing};
use crate::codec::types::acurast::attestation::Attestation;
use crate::codec::types::acurast::job_registration::JobRegistration;
use crate::codec::types::acurast::{
    job_environment::JobEnvironment, job_identifier::JobIdentifier,
};
use crate::codec::types::job_data::JobData;
use crate::codec::types::marketplace::job_assigment::JobAssignment;
use crate::codec::types::metrics::MetricPool;
use crate::codec::types::processor_overview::ProcessorOverview;
use crate::codec::types::uniques::pallet_uniques_item_details::PalletUniquesItemDetails;
use crate::engine::pallet::state::State;
use crate::types::system::FrameSystemAccountInfo;
use crate::Result;
use crate::RpcEngine;
use async_trait::async_trait;
use std::sync::Arc;

pub struct V1AcurastStorageImpl {
    base: V0AcurastStorageImpl,
}

impl V1AcurastStorageImpl {
    pub fn new(engine: Arc<dyn RpcEngine>, state: Arc<State>) -> Self {
        let v0 = V0AcurastStorageImpl::new(engine, state, 1);
        Self { base: v0 }
    }
}

#[async_trait]
impl VXAcurastStorage for V1AcurastStorageImpl {
    async fn get_manager_identifier(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<i32> {
        self.base
            .get_manager_identifier(account_id, block_hash, timeout)
            .await
    }

    async fn get_manager_counter(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<u64> {
        self.base
            .get_manager_counter(account_id, block_hash, timeout)
            .await
    }

    async fn get_asset(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<PalletUniquesItemDetails> {
        self.base.get_asset(manager_id, block_hash, timeout).await
    }

    async fn get_account_info(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> FrameSystemAccountInfo {
        self.base
            .get_account_info(account_id, block_hash, timeout)
            .await
    }

    async fn get_assigned_jobs(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Vec<JobAssignment> {
        self.base
            .get_assigned_jobs(account_id, block_hash, timeout)
            .await
    }

    async fn get_job_registration(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobRegistration> {
        self.base
            .get_job_registration(job_identifier, block_hash, timeout)
            .await
    }

    async fn get_job_data(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobData> {
        self.base
            .get_job_data(job_identifier, block_hash, timeout)
            .await
    }

    async fn get_job_environment(
        &self,
        job_identifier: JobIdentifier,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobEnvironment> {
        self.base
            .get_job_environment(job_identifier, account_id, block_hash, timeout)
            .await
    }

    async fn is_attested(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> bool {
        self.base.is_attested(account_id, block_hash, timeout).await
    }

    async fn get_attestation(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Attestation> {
        self.base
            .get_attestation(account_id, block_hash, timeout)
            .await
    }

    async fn get_marketplace_pricing(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<MarketplacePricing> {
        self.base
            .get_marketplace_pricing(account_id, block_hash, timeout)
            .await
    }

    async fn get_marketplace_advertisement_restriction(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<MarketplaceAdvertisementRestriction> {
        self.base
            .get_marketplace_advertisement_restriction(account_id, block_hash, timeout)
            .await
    }

    async fn get_metric_pool_ids(
        &self,
        names: Vec<String>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<Option<u8>>> {
        self.base
            .get_metric_pool_ids(names, block_hash, timeout)
            .await
    }

    async fn get_metric_pools(
        &self,
        ids: Vec<u8>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<Option<MetricPool>>> {
        self.base.get_metric_pools(ids, block_hash, timeout).await
    }

    async fn get_management_endpoint(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Vec<u8>> {
        self.base
            .get_management_endpoint(manager_id, block_hash, timeout)
            .await
    }

    async fn get_processor_overview(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<ProcessorOverview> {
        self.base
            .get_processor_overview(account_id, block_hash, timeout)
            .await
    }
}
