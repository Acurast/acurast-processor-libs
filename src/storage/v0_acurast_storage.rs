use async_trait::async_trait;
use blake2::Blake2bVar;
use blake2::digest::{Update, VariableOutput};
use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use std::io::Cursor;
use std::sync::Arc;
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
    ) -> Option<FrameSystemAccountInfo>;
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
    ) -> Option<FrameSystemAccountInfo> {
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(&xxh128_unsigned("System".as_bytes()));
        key.extend_from_slice(&xxh128_unsigned("Account".as_bytes()));
        key.extend_from_slice(&blake2b_128(account_id, 16));
        key.extend_from_slice(account_id);
        if let Ok(storage_option) = self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            if let Some(storage) = storage_option {
                let ba = hex_to_ba(&storage);
                return Some(FrameSystemAccountInfo::read_account_info(ba.as_slice()))
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