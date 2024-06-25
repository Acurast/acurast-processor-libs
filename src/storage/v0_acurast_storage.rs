use async_trait::async_trait;
use blake2::Blake2bVar;
use blake2::digest::{Update, VariableOutput};
use std::sync::Arc;
use crate::hex::hex_to_ba;
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
    ) -> Option<u32>;
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
    ) -> Option<u32> {
        let mut key = Vec::new();
        key.extend_from_slice(&xxh128_unsigned("AcurastProcessorManager".as_bytes()));
        key.extend_from_slice(&xxh128_unsigned("ProcessorToManagerIdIndex".as_bytes()));
        key.extend_from_slice(&blake2b_128(account_id, 16));
        //let unsigned = i8_slice_to_u8_slice(key.as_slice());
        //println!("get_manager_identifier AcurastProcessorManager: {:?}",&blake2b_128(account_id).to_le_bytes());
        println!("get_manager_identifier AcurastProcessorManager: {:?}",&xxh128_signed("AcurastProcessorManager".as_bytes()));
        println!("get_manager_identifier ProcessorToManagerIdIndex: {:?}",&xxh128_signed("ProcessorToManagerIdIndex".as_bytes()));
        //println!("get_manager_identifier key: {:?}",&xxh128_signed("AcurastProcessorManager".as_bytes()));
        println!("get_manager_identifier key: {:?}",key);
        if let Ok(storage_option) = self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            if let Some(storage) = storage_option {
                let ba = hex_to_ba(&storage);
                let z = u32::from_be_bytes(ba, 16);
                if let Ok(result) = z {
                    return Some(result);
                }
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