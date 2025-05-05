use crate::big_uint::BigIntToInt;
use crate::codec::types::acurast::attestation::Attestation;
use crate::codec::types::acurast::job_environment::JobEnvironment;
use crate::codec::types::acurast::job_identifier::JobIdentifier;
use crate::codec::types::acurast::job_registration::JobRegistration;
use crate::codec::types::marketplace::job_assigment::JobAssignment;
use crate::codec::types::metrics::MetricPool;
use crate::codec::types::processor_overview::ProcessorOverview;
use crate::codec::types::uniques::pallet_uniques_item_details::PalletUniquesItemDetails;
use crate::cursor::{read_byte_array, read_changes_opt};
use crate::engine::pallet::state::State;
use crate::hex::{hex_to_ba, read_u128, read_u64, ToHex};
use crate::storage::acurast_storage::StorageKey;
use crate::types::system::FrameSystemAccountInfo;
use crate::u8a::biguint_to_u8a;
use crate::RpcEngine;
use crate::{AcurastError, Result};
use async_trait::async_trait;
use byteorder::ReadBytesExt;
use num_bigint::BigUint;
use num_traits::Zero;
use std::io::Cursor;
use std::sync::Arc;

#[uniffi::export]
#[async_trait]
pub trait VXAcurastStorage: Send + Sync {
    async fn get_manager_identifier(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<i32>;

    async fn get_manager_counter(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<u64>;

    async fn get_asset(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<PalletUniquesItemDetails>;

    async fn get_account_info(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> FrameSystemAccountInfo;

    async fn get_assigned_jobs(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Vec<JobAssignment>;

    async fn get_job_registration(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobRegistration>;

    async fn get_job_environment(
        &self,
        job_identifier: JobIdentifier,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobEnvironment>;

    async fn is_attested(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> bool;

    async fn get_attestation(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Attestation>;

    async fn get_metric_pool_ids(
        &self,
        names: Vec<String>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<Option<u8>>>;

    async fn get_metric_pools(
        &self,
        ids: Vec<u8>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<Option<MetricPool>>>;

    async fn get_management_endpoint(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Vec<u8>>;

    async fn get_processor_overview(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<ProcessorOverview>;
}

pub struct V0AcurastStorageImpl {
    engine: Arc<dyn RpcEngine>,
    state: Arc<State>,
    version: u32,
}

impl V0AcurastStorageImpl {
    pub fn new(engine: Arc<dyn RpcEngine>, state: Arc<State>, version: u32) -> Self {
        Self {
            engine,
            state,
            version,
        }
    }
}

#[async_trait]
impl VXAcurastStorage for V0AcurastStorageImpl {
    async fn get_manager_identifier(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<i32> {
        let key = acurast_processor_manager_processor_to_manager_id_index(account_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let value_u128 = read_u128(&ba);
            let value_i32 = value_u128 as i32;
            return Some(value_i32);
        }
        None
    }

    async fn get_manager_counter(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<u64> {
        let key = acurast_processor_manager_manager_counter(account_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let value_u64 = read_u64(&ba);
            return Some(value_u64);
        }
        None
    }

    async fn get_asset(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<PalletUniquesItemDetails> {
        let key = uniques_asset(manager_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let mut cursor = Cursor::new(&ba[..]);
            return Some(PalletUniquesItemDetails::read(&mut cursor));
        }
        None
    }

    async fn get_account_info(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> FrameSystemAccountInfo {
        let key = system_account(account_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            return FrameSystemAccountInfo::read_account_info(ba.as_slice());
        }
        FrameSystemAccountInfo::empty()
    }

    async fn get_assigned_jobs(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Vec<JobAssignment> {
        let key = acurast_marketplace_stored_matches(account_id);

        let mut jobs: Vec<JobAssignment> = Vec::new();
        log::debug!("get_assigned_jobs: {:?}", account_id);
        let keys = self
            .state
            .get_keys(&key, block_hash, timeout, &self.engine)
            .await
            .unwrap_or(Vec::new());
        if let Ok(storage) = self
            .state
            .get_storage_at(keys, None, None, &self.engine)
            .await
        {
            log::debug!("get_assigned_jobs {:?}", storage);
            if !storage.is_empty() {
                for change in &storage[0].changes {
                    let job_assignment = JobAssignment::read(
                        change
                            .iter()
                            .filter_map(|c| c.as_ref().map(|s| s.as_str()))
                            .collect(),
                        self.version,
                    )
                    .unwrap();
                    jobs.push(job_assignment);
                    /*if let Ok(job_assignment) = JobAssignment::read(change.iter().map(|s| s.as_str()).collect(), 0) {
                        jobs.push(job_assignment);
                    }*/
                }
            }
            return jobs;
        }
        jobs
    }

    async fn get_job_registration(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobRegistration> {
        let key = acurast_stored_job_registration(&job_identifier);

        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let mut cursor = Cursor::new(ba);
            return Some(JobRegistration::read(&mut cursor, self.version).unwrap());
        }
        None
    }

    async fn get_job_environment(
        &self,
        job_identifier: JobIdentifier,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<JobEnvironment> {
        let key = acurast_execution_environment(&job_identifier, account_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let mut cursor = Cursor::new(ba);
            return Some(JobEnvironment::read(&mut cursor, self.version).unwrap());
        }
        None
    }

    async fn is_attested(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> bool {
        let key = acurast_stored_attestation(account_id);
        log::debug!("is_attested: {:?}", account_id);
        let keys = self
            .state
            .get_keys(&key, block_hash, timeout, &self.engine)
            .await
            .unwrap_or(Vec::new());
        if let Ok(storage) = self
            .state
            .get_storage_at(keys, None, None, &self.engine)
            .await
        {
            log::debug!("is_attested {:?}", storage);
            return !storage.is_empty();
        }
        false
    }

    async fn get_attestation(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Attestation> {
        let key = acurast_stored_attestation(account_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let mut cursor = Cursor::new(ba);
            return Some(Attestation::read(&mut cursor).unwrap());
        }
        None
    }

    async fn get_metric_pool_ids(
        &self,
        names: Vec<String>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<Option<u8>>> {
        let key = acurast_compute_metric_pool_lookup();
        let keys: Vec<String> = names
            .into_iter()
            .map(|name| {
                let mut bytes = name.into_bytes();
                bytes.resize(24, 0);
                let combined = [key.clone(), bytes].concat();
                combined.to_hex()
            })
            .collect();

        let mut pool_ids: Vec<Option<u8>> = Vec::new();
        log::debug!("get_metric_pool_ids keys: {:?}", keys);

        if let Ok(storage) = self
            .state
            .get_storage_at(keys, block_hash, timeout, &self.engine)
            .await
        {
            if !storage.is_empty() {
                let changes = storage.into_iter().nth(0).unwrap().changes;
                for change in changes {
                    let hex_str = change
                        .into_iter()
                        .nth(1)
                        .flatten()
                        .ok_or(AcurastError::ParsingError)?;
                    if let Ok(bytes) = hex::decode(hex_str) {
                        let mut cursor = Cursor::new(bytes);
                        let byte = cursor.read_u8().map_err(|_| AcurastError::ParsingError)?;
                        pool_ids.push(Some(byte));
                    } else {
                        pool_ids.push(None);
                    }
                }
            }
        } else {
            log::debug!("get_metric_pool_ids failed");
        }
        Ok(pool_ids)
    }

    async fn get_metric_pools(
        &self,
        ids: Vec<u8>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<Option<MetricPool>>> {
        let key = acurast_compute_metric_pools();
        let keys: Vec<String> = ids
            .into_iter()
            .map(|id| {
                let combined = [key.clone(), vec![id]].concat();
                combined.to_hex()
            })
            .collect();
        let mut metrics: Vec<Option<MetricPool>> = Vec::new();
        if let Ok(storage) = self
            .state
            .get_storage_at(keys, block_hash, timeout, &self.engine)
            .await
        {
            if !storage.is_empty() {
                let changes = storage.into_iter().nth(0).unwrap().changes;
                for change in changes {
                    let hex_str = change
                        .into_iter()
                        .nth(1)
                        .flatten()
                        .ok_or(AcurastError::ParsingError)?;
                    if let Ok(bytes) = hex::decode(hex_str) {
                        let mut cursor = Cursor::new(bytes);
                        let metric = MetricPool::read(&mut cursor)
                            .map_err(|_| AcurastError::ParsingError)?;
                        metrics.push(Some(metric));
                    } else {
                        metrics.push(None);
                    }
                }
            }
        } else {
            log::debug!("get_metric_pools failed");
        }
        Ok(metrics)
    }

    async fn get_management_endpoint(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Vec<u8>> {
        let key = acurast_processor_manager_management_endpoint(manager_id);
        let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        else {
            return None;
        };

        let bytes = hex_to_ba(&storage);
        let mut cursor = Cursor::new(bytes);

        return read_byte_array(&mut cursor).ok();
    }

    async fn get_processor_overview(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<ProcessorOverview> {
        let manager_id_key = acurast_processor_manager_processor_to_manager_id_index(account_id);
        let attestation_key = acurast_stored_attestation(account_id);
        let heartbeat_key = acurast_processor_manager_processor_heartbeat(account_id);

        let keys = vec![
            hex::encode(manager_id_key),
            hex::encode(attestation_key),
            hex::encode(heartbeat_key),
        ];

        let Ok(storage) = self
            .state
            .get_storage_at(keys, block_hash, timeout, &self.engine)
            .await
        else {
            return None;
        };

        let Some(changes) = storage.first().map(|r| &r.changes) else {
            return Some(Default::default());
        };

        let manager_identifier = read_changes_opt(changes, 0, |bytes| read_u128(&bytes) as i32);
        let is_attested = read_changes_opt(changes, 1, |_| {}).is_some();
        let last_heartbeat_timestamp =
            read_changes_opt(changes, 2, |bytes| read_u128(&bytes) as u64);

        Some(ProcessorOverview {
            manager_identifier,
            is_attested,
            last_heartbeat_timestamp,
        })
    }
}

fn acurast_stored_job_registration(job_identifier: &JobIdentifier) -> Vec<u8> {
    let origin = job_identifier.origin.to_u8a();
    let id = job_identifier.id.to_u8a();

    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix("Acurast", "StoredJobRegistration");
    key.extend_with_blake2_128concat(&origin);
    key.extend_with_blake2_128concat(&id);

    key
}

fn acurast_execution_environment(job_identifier: &JobIdentifier, account_id: &[u8]) -> Vec<u8> {
    let mut job = job_identifier.origin.to_u8a();
    job.extend(job_identifier.id.to_u8a());

    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix("Acurast", "ExecutionEnvironment");
    key.extend_with_blake2_128concat(&job);
    key.extend_with_blake2_128concat(account_id);

    key
}

fn acurast_stored_attestation(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix("Acurast", "StoredAttestation");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn acurast_marketplace_stored_matches(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix("AcurastMarketplace", "StoredMatches");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn acurast_processor_manager_processor_to_manager_id_index(account_id: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix("AcurastProcessorManager", "ProcessorToManagerIdIndex");
    key.extend_with_blake2_128(account_id);

    key
}

fn acurast_processor_manager_manager_counter(account_id: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix("AcurastProcessorManager", "ManagerCounter");
    key.extend_with_blake2_128(account_id);

    key
}

fn acurast_processor_manager_management_endpoint(manager_id: i32) -> Vec<u8> {
    let manager_id = &biguint_to_u8a(&BigUint::from(manager_id as u32));

    let mut key = Vec::new();
    key.extend_with_storage_prefix("AcurastProcessorManager", "ManagementEndpoint");
    key.extend_with_blake2_128concat(manager_id);

    key
}

fn acurast_processor_manager_processor_heartbeat(account_id: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix("AcurastProcessorManager", "ProcessorHeartbeat");
    key.extend_with_blake2_128(account_id);

    key
}

fn acurast_compute_metric_pool_lookup() -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix("AcurastCompute", "MetricPoolLookup");

    key
}

fn acurast_compute_metric_pools() -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix("AcurastCompute", "MetricPools");

    key
}

fn system_account(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix("System", "Account");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn uniques_asset(manager_id: i32) -> Vec<u8> {
    let manager_id = &biguint_to_u8a(&BigUint::from(manager_id as u32));
    let collection_id = &biguint_to_u8a(&BigUint::zero());
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix("Uniques", "Asset");
    key.extend_with_blake2_128concat(collection_id);
    key.extend_with_blake2_128concat(manager_id);

    key
}

#[cfg(test)]
mod tests {
    use crate::codec::types::acurast::job_identifier::{Kind, MultiOrigin};

    use super::*;

    #[test]
    fn test_acurast_stored_job_registration() {
        let key = acurast_stored_job_registration(&JobIdentifier {
            origin: MultiOrigin {
                kind: Kind::Acurast,
                source: hex_to_ba(
                    "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
                ),
            },
            id: BigUint::from(0u32),
        });

        assert_eq!(
            key,
            hex_to_ba("0xd8f45172ad1e7575680eaa8157102800f86e394d601279aa535d660453ff8e09ff699fe6ae26ef97168ddeaecbc34ce300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d463be1d58a72e9618ea59884367c435800000000000000000000000000000000")
        );
    }

    #[test]
    fn test_acurast_execution_environment() {
        let key = acurast_execution_environment(
            &JobIdentifier {
                origin: MultiOrigin {
                    kind: Kind::Acurast,
                    source: hex_to_ba(
                        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
                    ),
                },
                id: BigUint::from(0u32),
            },
            &hex_to_ba("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
        );

        assert_eq!(
            key,
            hex_to_ba("0xd8f45172ad1e7575680eaa81571028003b13796cad693cd980d0f24b9ff4cbfaea6c62216b6983bf26def5438482d4ba00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00000000000000000000000000000000de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
        );
    }

    #[test]
    fn test_acurast_stored_attestation() {
        let key = acurast_stored_attestation(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0xd8f45172ad1e7575680eaa8157102800f75db0c33624f81fd193acdc07620654de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
        );
    }

    #[test]
    fn test_acurast_marketplace_stored_matches() {
        let key = acurast_marketplace_stored_matches(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0x1aee6710ac79060b1e13291ba85112af2b949d1a72012eeaa1f6b481830d0d73de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
        )
    }

    #[test]
    fn test_acurast_processor_manager_processor_to_manager_id_index() {
        let key = acurast_processor_manager_processor_to_manager_id_index(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0xc698b31ad72d251a494de23e0dd66846f3a728ad8efdeffd18f039142abe31b2de1e86a9a8c739864cf3cc5ec2bea59f")
        );
    }

    #[test]
    fn test_acurast_processor_manager_manager_counter() {
        let key = acurast_processor_manager_manager_counter(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0xc698b31ad72d251a494de23e0dd6684652af9a5a36f7ce16cc96808f97171a0bde1e86a9a8c739864cf3cc5ec2bea59f")
        );
    }

    #[test]
    fn test_acurast_processor_manager_management_endpoint() {
        let key = acurast_processor_manager_management_endpoint(0);

        assert_eq!(
            key,
            hex_to_ba("0xc698b31ad72d251a494de23e0dd66846baf48d86cb51dd4c22ca340c99164aaa463be1d58a72e9618ea59884367c435800000000000000000000000000000000")
        );
    }

    #[test]
    fn test_acurast_processor_manager_processor_heartbeat() {
        let key = acurast_processor_manager_processor_heartbeat(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0xc698b31ad72d251a494de23e0dd6684690ede7a772410f47186f66100c0bf9ecde1e86a9a8c739864cf3cc5ec2bea59f")
        )
    }

    #[test]
    fn test_acurast_compute_metric_pool_lookup() {
        let key = acurast_compute_metric_pool_lookup();

        assert_eq!(
            key,
            hex_to_ba("0x56c55973ec96db3a5ef84bbd48553aebe72eacd0a9cd6eeac082faf837bb0e1a")
        );
    }

    #[test]
    fn test_acurast_compute_metric_pools() {
        let key = acurast_compute_metric_pools();

        assert_eq!(
            key,
            hex_to_ba("0x56c55973ec96db3a5ef84bbd48553aebf8721c64ae517455ab3aaead1f107f79")
        );
    }

    #[test]
    fn test_system_account() {
        let key = system_account(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
        );
    }

    #[test]
    fn test_uniques_asset() {
        let key = uniques_asset(0);

        assert_eq!(
            key,
            hex_to_ba("0x5e8a19e3cd1b7c148b33880c479c0281d34371a193a751eea5883e9553457b2e463be1d58a72e9618ea59884367c435800000000000000000000000000000000463be1d58a72e9618ea59884367c435800000000000000000000000000000000")
        );
    }
}
