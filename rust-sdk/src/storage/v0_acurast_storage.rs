use crate::big_uint::BigIntToInt;
use crate::codec::types::account_overview::{AccountOverview, DelegationWithCommitment};
use crate::codec::types::acurast::advertisement::{MarketplaceAdvertisementRestriction, MarketplacePricing};
use crate::codec::types::acurast::attestation::Attestation;
use crate::codec::types::acurast::job_environment::JobEnvironment;
use crate::codec::types::acurast::job_identifier::JobIdentifier;
use crate::codec::types::acurast::job_registration::JobRegistration;
use crate::codec::types::compute::commitment::Commitment;
use crate::codec::types::compute::delegation::Delegation;
use crate::codec::types::compute::metrics::MetricPool;
use crate::codec::types::account_info::AccountInfo;
use crate::codec::types::job_data::JobData;
use crate::codec::types::management_data::ManagementData;
use crate::codec::types::marketplace::job_assigment::JobAssignment;
use crate::codec::types::processor_overview::ProcessorOverview;
use crate::codec::types::token_conversion::token_conversion::{self, TokenConversion};
use crate::codec::types::uniques::pallet_uniques_item_details::PalletUniquesItemDetails;
use crate::codec::types::vesting::vesting::Vesting;
use crate::cursor::{read_byte_array, read_changes_opt, read_list};
use crate::engine::pallet::state::State;
use crate::hex::{hex_to_ba, read_u128, read_u64, ToHex};
use crate::storage::acurast_storage::StorageKey;
use crate::u8a::biguint_to_u8a;
use crate::RpcEngine;
use crate::{AcurastError, Result};
use async_trait::async_trait;
use byteorder::ReadBytesExt;
use num_bigint::BigUint;
use num_traits::{ConstZero, One, Zero};
use serde_json::de;
use uniffi::deps::bytes::Buf;
use std::collections::HashMap;
use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;
use std::vec;

#[uniffi::export]
#[async_trait]
pub trait VXAcurastStorage: Send + Sync {
    async fn get_manager_identifier(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<i32>>;

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
    ) -> AccountInfo;

    async fn get_account_overview(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<AccountOverview>>;

    async fn get_assigned_jobs(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<JobAssignment>>;

    async fn get_job_registration(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobRegistration>>;

    async fn get_job_data(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobData>>;

    async fn get_job_environment(
        &self,
        job_identifier: JobIdentifier,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobEnvironment>>;

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
    ) -> Result<Option<Attestation>>;

    async fn get_marketplace_pricing(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<MarketplacePricing>>;

    async fn get_marketplace_advertisement_restriction(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<MarketplaceAdvertisementRestriction>>;

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

    async fn get_management_data(
        &self,
        manager_id: i32,
        manager_account_id: Option<Vec<u8>>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<ManagementData>>;

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
    ) -> Result<Option<i32>> {
        let key = acurast_processor_manager_processor_to_manager_id_index(account_id);
        if let Some(storage) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
            .map_err(|e| AcurastError::RuntimeError(format!("{}", e)))?
        {
            let ba = hex_to_ba(&storage);
            let value_u128 = read_u128(&ba);
            let value_i32 = value_u128 as i32;
            return Ok(Some(value_i32));
        }
        Ok(None)
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
    ) -> AccountInfo {
        let key = system_account(account_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            return AccountInfo::read_account_info(ba.as_slice());
        }
        AccountInfo::empty()
    }

    async fn get_account_overview(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<AccountOverview>> {
        let committer_id = self.get_committer_id(account_id, block_hash.clone(), timeout).await?;
        let Some((account_info, vesting, token_conversion, delegations)) = self.get_partial_account_overview(account_id, block_hash.clone(), timeout).await? else {
            return Ok(None);
        };
        let mut commitments: HashMap<BigUint, Commitment> = self.get_commitments(&committer_id, &delegations, block_hash, timeout).await?;
        let commitment = committer_id.map(|id| commitments.remove(&id)).flatten();
        let delegations: Vec<DelegationWithCommitment> = delegations
            .into_iter()
            .filter_map(|(committer_id, delegation)| {
                let Some(commitment) = commitments.remove(&committer_id) else {
                    return None;
                };

                Some(DelegationWithCommitment {
                    delegation,
                    commitment
                })
            })
            .collect();

        Ok(Some(AccountOverview {
            account_info,
            commitment,
            delegations,
            vesting,
            conversion: token_conversion,
        }))
    }

    async fn get_assigned_jobs(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Vec<JobAssignment>> {
        let key = acurast_marketplace_stored_matches(account_id);

        let mut jobs: Vec<JobAssignment> = Vec::new();
        log::debug!("get_assigned_jobs: {:?}", account_id);
        let keys = self
            .state
            .get_keys(&key, block_hash, timeout, &self.engine)
            .await?;
        let storage = self
            .state
            .get_storage_at(keys, None, None, &self.engine)
            .await?;
        log::debug!("get_assigned_jobs {:?}", storage);
        if !storage.is_empty() {
            for change in &storage[0].changes {
                let change: Vec<&str> = change
                    .iter()
                    .filter_map(|c| c.as_ref().map(|s| s.as_str()))
                    .collect();

                if change.len() < 2 {
                    continue;
                }

                let job_assignment = JobAssignment::read(
                    change,
                    self.version,
                )
                .map_err(|e| AcurastError::RuntimeError(e.to_string()))?;
                jobs.push(job_assignment);
            }
        }
        Ok(jobs)
    }

    async fn get_job_registration(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobRegistration>> {
        let key = acurast_stored_job_registration(&job_identifier);

        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let mut cursor = Cursor::new(ba);
            return JobRegistration::read(&mut cursor, self.version)
                .map(Some)
                .map_err(|e| AcurastError::RuntimeError(e.to_string()));
        }
        Ok(None)
    }

    async fn get_job_data(
        &self,
        job_identifier: JobIdentifier,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobData>> {
        let registration_key = acurast_stored_job_registration(&job_identifier);
        let key_id_key = acurast_marketplace_job_key_ids(&job_identifier);

        let keys = vec![
            hex::encode(registration_key),
            hex::encode(key_id_key),
        ];

        let Ok(storage) = self
            .state
            .get_storage_at(keys, block_hash, timeout, &self.engine)
            .await
        else {
            return Ok(None);
        };

        let Some(changes) = storage.first().map(|r| &r.changes) else {
            return Ok(None);
        };

        let Some(registration) = read_changes_opt(changes, 0, |bytes| {
            JobRegistration::read(&mut Cursor::new(bytes), self.version)
        })
        .transpose()
        .map_err(|e| AcurastError::RuntimeError(e.to_string()))?
        else {
            return Ok(None);
        };
        let key_id = read_changes_opt(changes, 1, |bytes| {
            bytes
        });

        Ok(Some(JobData {
            registration,
            key_id,
        }))
    }

    async fn get_job_environment(
        &self,
        job_identifier: JobIdentifier,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<JobEnvironment>> {
        let key = acurast_execution_environment(&job_identifier, account_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let mut cursor = Cursor::new(ba);
            return JobEnvironment::read(&mut cursor, self.version)
                .map(Some)
                .map_err(|e| AcurastError::RuntimeError(e.to_string()));
        }
        Ok(None)
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
    ) -> Result<Option<Attestation>> {
        let key = acurast_stored_attestation(account_id);
        if let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            let ba = hex_to_ba(&storage);
            let mut cursor = Cursor::new(ba);
            return Attestation::read(&mut cursor)
                .map(Some)
                .map_err(|e| AcurastError::RuntimeError(e.to_string()));
        }
        Ok(None)
    }

    async fn get_marketplace_pricing(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<MarketplacePricing>> {
        let key = acurast_marketplace_stored_advertisement_pricing(account_id);
        let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        else {
            return Ok(None);
        };

        let ba = hex_to_ba(&storage);
        let mut cursor = Cursor::new(ba);
        return MarketplacePricing::read(&mut cursor)
            .map(Some)
            .map_err(|e| AcurastError::RuntimeError(e.to_string()));
    }

    async fn get_marketplace_advertisement_restriction(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<MarketplaceAdvertisementRestriction>> {
        let key = acurast_marketplace_stored_advertisement_restriction(account_id);
        let Ok(Some(storage)) = self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        else {
            return Ok(None);
        };

        let ba = hex_to_ba(&storage);
        let mut cursor = Cursor::new(ba);
        return MarketplaceAdvertisementRestriction::read(&mut cursor)
            .map(Some)
            .map_err(|e| AcurastError::RuntimeError(e.to_string()));
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
                    let Some(hex_str) = change.into_iter().nth(1).flatten() else {
                        pool_ids.push(None);
                        continue;
                    };
                    let hex_str = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
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
                    let hex_str = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
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

    async fn get_management_data(
        &self,
        manager_id: i32,
        manager_account_id: Option<Vec<u8>>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<ManagementData>> {
        let management_endpoint_key = acurast_processor_manager_management_endpoint(manager_id);
        let migration_data_key = manager_account_id.map(|account_id| acurast_processor_manager_processor_migration_data(&account_id));

        let mut keys = vec![
            hex::encode(management_endpoint_key),
        ];
        if let Some(migration_data_key) = migration_data_key {
            keys.push(hex::encode(migration_data_key));
        }

        let storage = self
            .state
            .get_storage_at(keys, block_hash, timeout, &self.engine)
            .await
            .map_err(|e| AcurastError::RuntimeError(format!("{}", e)))?;

        let Some(changes) = storage.first().map(|r| &r.changes) else {
            return Ok(None);
        };

        let management_endpoint = read_changes_opt(changes, 0, |bytes| {
            read_byte_array(&mut Cursor::new(bytes)).ok()
        });
        let migration_data = read_changes_opt(changes, 1, |bytes| {
            bytes
        });

        return Ok(Some(ManagementData {
            management_endpoint: management_endpoint.unwrap_or(None),
            migration_proof: migration_data,
        }))
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

type PartialAccountOverview = (AccountInfo, Vec<Vesting>, Option<TokenConversion>, Vec<(BigUint, Delegation)>);

impl V0AcurastStorageImpl {
    async fn get_committer_id(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<BigUint>> {
        let committer_id_partial_key = uniques_account(account_id, BigUint::one());
        let committer_id_keys = self
            .state
            .get_keys(&committer_id_partial_key, block_hash.clone(), timeout, &self.engine)
            .await
            .map_err(|e| AcurastError::RuntimeError(e.to_string()))?;
        let committer_id = committer_id_keys
            .first()
            .map(|key| {
                let mut cursor = Cursor::new(hex_to_ba(key));
                cursor.set_position((committer_id_partial_key.len() + 16 /* blake2_128(u128).size */).try_into()?);
                crate::cursor::read_u128(&mut cursor)
            })
            .transpose()
            .map_err(|e| AcurastError::RuntimeError(e.to_string()))?;

        Ok(committer_id)
    }

    async fn get_partial_account_overview(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<PartialAccountOverview>> {
        let delegation_partial_key = acurast_compute_delegations(account_id, None);
        let delegation_keys = self
            .state
            .get_keys(&delegation_partial_key, block_hash.clone(), timeout, &self.engine)
            .await
            .map_err(|e| AcurastError::RuntimeError(e.to_string()))?;
        let system_account_key = system_account(account_id);
        let vesting_key = vesting_vesting(account_id);
        let token_conversion_key = acurast_token_conversion_locked_conversion(account_id);
        let keys = vec![
            vec![
                hex::encode(system_account_key),
                hex::encode(vesting_key),
                hex::encode(token_conversion_key),
            ],
            delegation_keys,
        ].concat();

        let storage = self
            .state
            .get_storage_at(keys, block_hash.clone(), timeout, &self.engine)
            .await
            .map_err(|e| AcurastError::RuntimeError(e.to_string()))?;
        let Some(changes) = storage.first().map(|r| &r.changes) else {
            return Ok(None);
        };
        let Some(account_info) = read_changes_opt(changes, 0, |bytes| {
            AccountInfo::read_account_info(&bytes)
        }) else {
            return Ok(None);
        };
        let vesting = read_changes_opt(changes, 1, |bytes| {
            let mut cursor = Cursor::new(bytes);
            read_list(&mut cursor, |c| Vesting::read(c))
        }).transpose().map_err(|e| AcurastError::RuntimeError(e.to_string()))?;
        let token_conversion = read_changes_opt(changes, 2, |bytes| {
            TokenConversion::read(&mut Cursor::new(bytes))
        }).transpose().map_err(|e| AcurastError::RuntimeError(e.to_string()))?;
        let mut delegations: Vec<(BigUint, Delegation)> = vec![];
        for change in changes[3..].iter() {
            let committer_id = if let Some(Some(key)) = change.get(0) {
                let committer_id = read_committer_id_from_key(key)?;
                Some(committer_id)
            } else {
                None
            };
            let delegation = if let Some(Some(value)) = change.get(1) {
                let delegation = Delegation::read(&mut Cursor::new(hex_to_ba(value))).map_err(|e| AcurastError::RuntimeError(e.to_string()))?;
                Some(delegation)
            } else {
                None
            };

            if let (Some(committer_id), Some(delegation)) = (committer_id, delegation) {
                delegations.push((committer_id, delegation));
            }
        }

        Ok(Some((account_info, vesting.unwrap_or_default(), token_conversion, delegations)))
    }

    async fn get_commitments(
        &self,
        committer_id: &Option<BigUint>,
        delegations: &Vec<(BigUint, Delegation)>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<HashMap<BigUint, Commitment>> {
        let mut keys = vec![];
        if let Some(committer_id) = committer_id {
            keys.push(hex::encode(acurast_compute_commitments(committer_id)));
        }
        for delegation in delegations.iter() {
            keys.push(hex::encode(acurast_compute_commitments(&delegation.0)));
        }

        let storage = self
            .state
            .get_storage_at(keys, block_hash.clone(), timeout, &self.engine)
            .await
            .map_err(|e| AcurastError::RuntimeError(e.to_string()))?;

        let mut commitments: HashMap<BigUint, Commitment> = HashMap::new();
        if let Some(changes) = storage.first().map(|r| &r.changes) {
            for change in changes {
                let committer_id = if let Some(Some(key)) = change.get(0) {
                    let committer_id = read_committer_id_from_key(key)?;
                    Some(committer_id)
                } else {
                    None
                };
                let commitment = if let Some(Some(value)) = change.get(1) {
                    let commitment = Commitment::read(&mut Cursor::new(hex_to_ba(value))).map_err(|e| AcurastError::RuntimeError(e.to_string()))?;
                    Some(commitment)
                } else {
                    None
                };

                if let (Some(committer_id), Some(commitment)) = (committer_id, commitment) {
                    commitments.insert(committer_id, commitment);
                }
            }
        }

        Ok(commitments)
    }
}

fn read_committer_id_from_key(key: &str) -> Result<BigUint> {
    let mut cursor = Cursor::new(hex_to_ba(key));
    let new_position: u64 = (cursor.remaining() - 16).try_into().map_err(|_| AcurastError::RuntimeError(String::from("Failed to convert remaining bytes to u64")))?;
    cursor.set_position(new_position);
    let committer_id = crate::cursor::read_u128(&mut cursor).map_err(|e| AcurastError::RuntimeError(e.to_string()))?;

    Ok(committer_id)
}

const PALLET_ACURAST: &str = "Acurast";
const PALLET_ACURAST_MARKETPLACE: &str = "AcurastMarketplace";
const PALLET_ACURAST_PROCESSOR_MANAGER: &str = "AcurastProcessorManager";
const PALLET_ACURAST_COMPUTE: &str = "AcurastCompute";
const PALLET_ACURAST_TOKEN_CONVERSION: &str = "AcurastTokenConversion";
const PALLET_SYSTEM: &str = "System";
const PALLET_UNIQUES: &str = "Uniques";
const PALLET_VESTING: &str = "Vesting";

fn acurast_stored_job_registration(job_identifier: &JobIdentifier) -> Vec<u8> {
    let origin = job_identifier.origin.to_u8a();
    let id = job_identifier.id.to_u8a();

    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST, "StoredJobRegistration");
    key.extend_with_blake2_128concat(&origin);
    key.extend_with_blake2_128concat(&id);

    key
}

fn acurast_execution_environment(job_identifier: &JobIdentifier, account_id: &[u8]) -> Vec<u8> {
    let mut job = job_identifier.origin.to_u8a();
    job.extend(job_identifier.id.to_u8a());

    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST, "ExecutionEnvironment");
    key.extend_with_blake2_128concat(&job);
    key.extend_with_blake2_128concat(account_id);

    key
}

fn acurast_stored_attestation(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST, "StoredAttestation");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn acurast_marketplace_stored_matches(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_MARKETPLACE, "StoredMatches");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn acurast_marketplace_job_key_ids(job_identifier: &JobIdentifier) -> Vec<u8> {
    let mut job = job_identifier.origin.to_u8a();
    job.extend(job_identifier.id.to_u8a());

    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_MARKETPLACE, "JobKeyIds");
    key.extend_with_blake2_128concat(&job);

    key
}

fn acurast_marketplace_stored_advertisement_pricing(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_MARKETPLACE, "StoredAdvertisementPricing");
    key.extend_with_blake2_128(account_id);

    key
}

fn acurast_marketplace_stored_advertisement_restriction(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_MARKETPLACE, "StoredAdvertisementRestriction");
    key.extend_with_blake2_128(account_id);

    key
}

fn acurast_processor_manager_processor_to_manager_id_index(account_id: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_PROCESSOR_MANAGER, "ProcessorToManagerIdIndex");
    key.extend_with_blake2_128(account_id);

    key
}

fn acurast_processor_manager_manager_counter(account_id: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_PROCESSOR_MANAGER, "ManagerCounter");
    key.extend_with_blake2_128(account_id);

    key
}

fn acurast_processor_manager_management_endpoint(manager_id: i32) -> Vec<u8> {
    let manager_id = &biguint_to_u8a(&BigUint::from(manager_id as u32));

    let mut key = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_PROCESSOR_MANAGER, "ManagementEndpoint");
    key.extend_with_blake2_128concat(manager_id);

    key
}

fn acurast_processor_manager_processor_heartbeat(account_id: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_PROCESSOR_MANAGER, "ProcessorHeartbeat");
    key.extend_with_blake2_128(account_id);

    key
}

fn acurast_processor_manager_processor_migration_data(account_id: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_PROCESSOR_MANAGER, "ProcessorMigrationData");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn acurast_compute_commitments(committer_id: &BigUint) -> Vec<u8> {
    let committer_id = &biguint_to_u8a(committer_id);

    let mut key = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_COMPUTE, "Commitments");
    key.extend_from_slice(committer_id);

    key
}

fn acurast_compute_delegations(account_id: &[u8], committer_id: Option<&BigUint>) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_COMPUTE, "Delegations");
    key.extend_with_xxh64concat(account_id);
    if let Some(committer_id) = committer_id {
        let committer_id = &biguint_to_u8a(committer_id);
        key.extend_from_slice(committer_id);
    }

    key
}

fn acurast_compute_metric_pool_lookup() -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_COMPUTE, "MetricPoolLookup");

    key
}

fn acurast_compute_metric_pools() -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_COMPUTE, "MetricPools");

    key
}

fn system_account(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_SYSTEM, "Account");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn uniques_asset(account_id: i32) -> Vec<u8> {
    let manager_id = &biguint_to_u8a(&BigUint::from(account_id as u32));
    let collection_id = &biguint_to_u8a(&BigUint::zero());
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_UNIQUES, "Asset");
    key.extend_with_blake2_128concat(collection_id);
    key.extend_with_blake2_128concat(manager_id);

    key
}

fn uniques_account(account_id: &[u8], collection_id: BigUint) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_UNIQUES, "Account");
    key.extend_with_blake2_128concat(account_id);
    key.extend_with_blake2_128concat(&biguint_to_u8a(&collection_id));

    key
}

fn acurast_token_conversion_locked_conversion(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_ACURAST_TOKEN_CONVERSION, "LockedConversion");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn vesting_vesting(account_id: &[u8]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::new();
    key.extend_with_storage_prefix(PALLET_VESTING, "Vesting");
    key.extend_with_blake2_128concat(account_id);

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
    fn test_acurast_marketplace_stored_advertisement_pricing() {
        let key = acurast_marketplace_stored_advertisement_pricing(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0x1aee6710ac79060b1e13291ba85112af7343c819ecdbbce7ea9c22c60dd68245de1e86a9a8c739864cf3cc5ec2bea59f")
        );
    }

    #[test]
    fn test_acurast_marketplace_stored_advertisement_restriction() {
        let key = acurast_marketplace_stored_advertisement_restriction(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0x1aee6710ac79060b1e13291ba85112af1d44a43f698cab34946cf754d09c275ade1e86a9a8c739864cf3cc5ec2bea59f")
        );
    }

    #[test]
    fn test_acurast_marketplace_job_key_ids() {
        let key = acurast_marketplace_job_key_ids(&JobIdentifier {
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
            hex_to_ba("0x1aee6710ac79060b1e13291ba85112afaf69a8f6416d983b30440e56737bd8f5ea6c62216b6983bf26def5438482d4ba00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00000000000000000000000000000000"),
        );
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
    fn test_acurast_processor_manager_processor_migration_data() {
        let key = acurast_processor_manager_processor_migration_data(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0xc698b31ad72d251a494de23e0dd66846c47b168bf86f71f46d2cc03079ff1ae4de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
        )
    }

    #[test]
    fn test_acurast_compute_commitments() {
        let key = acurast_compute_commitments(&BigUint::zero());

        assert_eq!(
            key,
            hex_to_ba("0x56c55973ec96db3a5ef84bbd48553aebca407206ec1ab726b2636c4b145ac28700000000000000000000000000000000")
        );
    }

    #[test]
    fn test_acurast_compute_delegations() {
        let key = acurast_compute_delegations(
            &hex_to_ba("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
            Some(&BigUint::zero()),
        );

        assert_eq!(
            key,
            hex_to_ba("0x56c55973ec96db3a5ef84bbd48553aebf6eed5c65f50198f0f53f8f499f77091518366b5b1bc7c99d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00000000000000000000000000000000")
        );

        let key = acurast_compute_delegations(
            &hex_to_ba("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
            None,
        );

        assert_eq!(
            key,
            hex_to_ba("0x56c55973ec96db3a5ef84bbd48553aebf6eed5c65f50198f0f53f8f499f77091518366b5b1bc7c99d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
        );
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

    #[test]
    fn test_uniques_account() {
        let key = uniques_account(
            &hex_to_ba("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
            BigUint::zero()
        );

        assert_eq!(
            key,
            hex_to_ba("0x5e8a19e3cd1b7c148b33880c479c0281b99d880ec681799c0cf30e8886371da9de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d463be1d58a72e9618ea59884367c435800000000000000000000000000000000")
        );
    }

    #[test]
    fn test_acurast_token_conversion_locked_conversion() {
        let key = acurast_token_conversion_locked_conversion(&hex_to_ba("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"));

        assert_eq!(
            key,
            hex_to_ba("0x668e0c9d8f6a62868eb8af40bb5b05b755c0f3d7e41584c39a4afabe7b681593de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
        );
    }

    #[test]
    fn test_vesting_vesting() {
        let key = vesting_vesting(&hex_to_ba("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"));

        assert_eq!(
            key,
            hex_to_ba("0x5f27b51b5ec208ee9cb25b55d87282435f27b51b5ec208ee9cb25b55d8728243de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
        );
    }
}
