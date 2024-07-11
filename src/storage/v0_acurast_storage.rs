use async_trait::async_trait;
use blake2::Blake2bVar;
use blake2::digest::{Update, VariableOutput};
use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use std::io::Cursor;
use std::sync::Arc;
use crate::big_uint::BigIntToInt;
use crate::codec::types::acurast::job_identifier::{self, JobIdentifier};
use crate::codec::types::acurast::job_registration::JobRegistration;
use crate::codec::types::marketplace::job_assigment::JobAssignment;
use crate::codec::types::uniques::pallet_uniques_item_details::PalletUniquesItemDetails;
use crate::hex::{hex_to_ba, read_u128};
use crate::types::system::{FrameSystemAccountInfo, FrameSystemAccountInfoData};
use crate::u8a::{biguint_to_u8a, ToU8a};
use crate::xxhash::{xxh128_signed, xxh128_unsigned};
use crate::vec_slice::i8_slice_to_u8_slice;
use crate::RpcEngine;
use crate::engine::pallet::state::State;

#[uniffi::export]
#[async_trait]
pub trait V0AcurastStorage: Send + Sync {
    async fn get_manager_identifier(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<i32>;

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
        &self, account_id: &[u8], 
        block_hash: Option<Vec<u8>>, 
        timeout: Option<u64>
    ) -> Vec<JobAssignment>;

    async fn get_job_registration(
        &self, 
        job_identifier: JobIdentifier, 
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>
    ) -> Option<JobRegistration>;
}

pub struct V0AcurastStorageImpl {
    engine: Arc<dyn RpcEngine>,
    state: Arc<State>,
}

impl V0AcurastStorageImpl {
    pub fn new(engine: Arc<dyn RpcEngine>, state: Arc<State>) -> Self {
        Self { engine, state }
    }
}

#[async_trait]
impl V0AcurastStorage for V0AcurastStorageImpl {
    async fn get_manager_identifier(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<i32> {
        let mut key = Vec::new();
        key.extend_from_slice(&xxh128_unsigned("AcurastProcessorManager".as_bytes()));
        key.extend_from_slice(&xxh128_unsigned("ProcessorToManagerIdIndex".as_bytes()));
        key.extend_from_slice(&blake2b_128(account_id, 16));
        if let Ok(storage_option) = self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            if let Some(storage) = storage_option {
                let ba = hex_to_ba(&storage);
                let value_u128 = read_u128(&ba);
                let value_i32 = value_u128 as i32;
                return Some(value_i32)
            }
        }
        None
    }

    async fn get_asset(
        &self,
        manager_id: i32,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<PalletUniquesItemDetails> {
        
        let manager = &biguint_to_u8a(&BigUint::from(manager_id as u32));
        let collection_id = &biguint_to_u8a(&BigUint::zero());
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(&xxh128_unsigned("Uniques".as_bytes()));
        key.extend_from_slice(&xxh128_unsigned("Asset".as_bytes()));
        key.extend_from_slice(&blake2b_128(collection_id, 16));
        key.extend_from_slice(collection_id);
        key.extend_from_slice(&blake2b_128(manager, 16));
        key.extend_from_slice(manager);
        if let Ok(storage_option) = self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            if let Some(storage) = storage_option {
                let ba = hex_to_ba(&storage);
                let mut cursor = Cursor::new(&ba[..]); 
                return Some(PalletUniquesItemDetails::read(&mut cursor))
            }
        }
        None
    }

    async fn get_account_info(
        &self,
        account_id: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> FrameSystemAccountInfo {
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(&xxh128_unsigned("System".as_bytes()));
        key.extend_from_slice(&xxh128_unsigned("Account".as_bytes()));
        key.extend_from_slice(&blake2b_128(account_id, 16));
        key.extend_from_slice(account_id);
        if let Ok(storage_option) = self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            if let Some(storage) = storage_option {
                let ba = hex_to_ba(&storage);
                return FrameSystemAccountInfo::read_account_info(ba.as_slice())
            }
        }
        FrameSystemAccountInfo::empty()
    }

    async fn get_assigned_jobs(
        &self, 
        account_id: &[u8], 
        block_hash: Option<Vec<u8>>, 
        timeout: Option<u64>
    ) -> Vec<JobAssignment> {
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(&xxh128_unsigned("AcurastMarketplace".as_bytes()));
        key.extend_from_slice(&xxh128_unsigned("StoredMatches".as_bytes()));
        key.extend_from_slice(&blake2b_128(account_id, 16));
        key.extend_from_slice(account_id);

        let mut jobs: Vec<JobAssignment> = Vec::new();
        println!("get_assigned_jobs: {:?}", account_id);
        let keys = self.state.get_keys(&key, block_hash, timeout, &self.engine).await.unwrap_or(Vec::new());
        if let Ok(storage) = self.state.get_storage_at(keys, None, None, &self.engine).await {
            println!("get_assigned_jobs {:?}", storage);
            if !storage.is_empty() {
                
                for change in &storage[0].changes {
                    let job_assignment = JobAssignment::read(change.iter().map(|s| s.as_str()).collect(), 0).unwrap();
                    jobs.push(job_assignment);
                    /*if let Ok(job_assignment) = JobAssignment::read(change.iter().map(|s| s.as_str()).collect(), 0) {
                        jobs.push(job_assignment);
                    }*/
                }
            }
            return jobs
        }
        jobs
    }

    async fn get_job_registration(
        &self, 
        job_identifier: JobIdentifier, 
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>
    ) -> Option<JobRegistration> {
        let mut key: Vec<u8> = Vec::new();
        let origin = job_identifier.origin.to_u8a();
        let jobId = job_identifier.id.to_u8a();
        key.extend_from_slice(&xxh128_unsigned("Acurast".as_bytes()));
        key.extend_from_slice(&xxh128_unsigned("StoredJobRegistration".as_bytes()));
        key.extend_from_slice(&blake2b_128(&origin, 16));
        key.extend_from_slice(&origin);
        key.extend_from_slice(&blake2b_128(&jobId, 16));
        key.extend_from_slice(&jobId);
        if let Ok(storage_option) = self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            if let Some(storage) = storage_option {
                let ba = hex_to_ba(&storage);
                let mut cursor = Cursor::new(ba);
                return Some(JobRegistration::read(&mut cursor, 0).unwrap())
            }
        }
        None
    }
}


/// Hashes the input data using BLAKE2b and returns a 128-bit hash as u128.
fn blake2b_128(input: &[u8], hash_size: usize) -> [u8; 16] {
    let mut hasher = Blake2bVar::new(hash_size).expect("Invalid hash size");
    hasher.update(input);
    let mut hash = vec![0u8; hash_size];
    hasher.finalize_variable(&mut hash).expect("Failed to finalize hash");

    // Convert to [i8; 16]
    /*let mut signed_hash = [0i8; 16];
    for (i, &byte) in hash.iter().enumerate() {
        signed_hash[i] = byte as i8;
    }*/
    let mut array = [0u8; 16];
    array.copy_from_slice(&hash);
    array
}