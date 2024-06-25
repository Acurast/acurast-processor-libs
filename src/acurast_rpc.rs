use async_trait::async_trait;
use blake2::{Blake2b, Digest};
use byteorder::ByteOrder;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use hex::{decode, FromHex};
use std::collections::HashMap;
use std::sync::Arc;
use xxhash_rust::xxh3::{Xxh3, xxh3_64_with_seed, xxh3_128};
use crate::hex::hex_to_ba;
use crate::storage::v0_acurast_storage::V0AcurastStorage;
use crate::storage::v0_acurast_storage::V0AcurastStorageImpl;
use std::io::{Cursor};
use crate::engine::engine::RpcEngine;
use crate::engine::pallet::author::Author;
use crate::engine::pallet::chain::Chain;
use crate::engine::pallet::state::{RuntimeVersion, State};
use crate::types::header::Header;
use crate::codec::processor_version::ProcessorVersion;
use crate::codec::processor_update_info::ProcessorUpdateInfo;

#[uniffi::export]
#[async_trait]
pub trait AcurastRpc: Send + Sync {
    async fn get_block_hash(&self, block_number: Option<i64>, timeout: Option<u64>) -> Option<String>;
    async fn get_header(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<Header>;
    async fn get_api_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> u32;
    async fn get_runtime_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> RuntimeVersion;
    async fn get_update_info(&self, account_id: Vec<u8>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<ProcessorUpdateInfo>;
    async fn get_known_binary_hash(&self, version: ProcessorVersion, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<Vec<u8>>;
    async fn submit_extrinsic(&self, extrinsic: Vec<u8>, timeout: Option<u64>) -> Option<String>;
    async fn call(&self, method: String, data: Option<Vec<u8>>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<String>;
    fn storage(&self, version: u32) -> Arc<dyn V0AcurastStorage>;
}

pub struct AcurastRpcImpl {
    engine: Arc<dyn RpcEngine>,
    state: Arc<State>,
    author: Arc<Author>,
    chain: Arc<Chain>,
    storages: HashMap<u32, Arc<dyn V0AcurastStorage>>,
}

impl AcurastRpcImpl {
    pub fn new(engine: Arc<dyn RpcEngine>) -> Self {
        let author = Arc::new(Author::new());
        let chain = Arc::new(Chain::new());
        let state = Arc::new(State::new());

        let storages: HashMap<u32, Arc<dyn V0AcurastStorage>> = vec![
            
            (0, Arc::new(V0AcurastStorageImpl::new(engine.clone(), state.clone())) as Arc<dyn V0AcurastStorage>),
            //(1, Arc::new(V1AcurastStorage::new(engine.clone(), state.clone())) as Arc<dyn AcurastStorage>),
        ].into_iter().collect();

        Self {
            engine,
            state,
            author,
            chain,
            storages,
        }
    }

    fn generate_key(prefix: &str, key: &str) -> Vec<u8> {
        let prefix_hash = xxh128(prefix.as_bytes(), 0);
        let key_hash = xxh128(key.as_bytes(), 0);
        [prefix_hash.to_le_bytes(), key_hash.to_le_bytes()].concat()
    }
    
}

#[async_trait]
impl AcurastRpc for AcurastRpcImpl {  
    async fn get_block_hash(&self, block_number: Option<i64>, timeout: Option<u64>) -> Option<String> {
        match self.chain.get_block_hash(block_number, timeout, &self.engine).await {
            Ok(block_hash) => { 
                Some(block_hash.unwrap()) 
            },
            Err(error) => { 
                None 
            }, // You can handle the error case here, e.g., log the error or return a default value
        }
    }

    async fn get_header(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<Header> {
        self.chain.get_header(block_hash, timeout, &self.engine).await.ok()
    }

    async fn get_api_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> u32 {
        let key = Self::generate_key("AcurastProcessorManager", "ApiVersion");
        match self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            Ok(storage_option) => {
                if let Some(storage) = storage_option {
                    let decoded_storage = decode(&storage).expect("Decoding failed");
                    let mut cursor = std::io::Cursor::new(decoded_storage);
                    cursor.read_u32::<BigEndian>().unwrap_or(0)

                } else {
                    0
                }
            },
            Err(_) => 0,
        }
    }

    async fn get_runtime_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> RuntimeVersion {
        self.state.get_runtime_version(block_hash, timeout, &self.engine).await.unwrap()
    }

    async fn get_update_info(&self, account_id: Vec<u8>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<ProcessorUpdateInfo> {
        let key = [
            Self::generate_key("AcurastProcessorManager", "ProcessorUpdateInfo"),
            //Self::hash_blake2b(&account_id),
            account_id.clone(),
        ].concat();

        match self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            Ok(storage_option) => {
                if let Some(storage) = storage_option {
                    let byte_array = hex_to_ba(&storage).unwrap();
                    let mut cursor = Cursor::new(&byte_array[..]); 
                    Some(ProcessorUpdateInfo::read(&mut cursor))
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }

    async fn get_known_binary_hash(&self, version: ProcessorVersion, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<Vec<u8>> {
        let version_bytes = version.to_u8a();
        let key = [
            Self::generate_key("AcurastProcessorManager", "KnownBinaryHash"),
            //Self::hash_blake2b(&version_bytes),
            version_bytes.clone(),
        ].concat();
        match self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            Ok(storage_option) => {
                if let Some(storage) = storage_option {
                    decode(&storage).ok()
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }

    async fn submit_extrinsic(&self, extrinsic: Vec<u8>, timeout: Option<u64>) -> Option<String> {
        self.author.submit_extrinsic(extrinsic, timeout, &self.engine).await.ok()
    }

    async fn call(&self, method: String, data: Option<Vec<u8>>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<String> {
        self.call(method, data, block_hash, timeout).await
    }

    fn storage(&self, version: u32) -> Arc<dyn V0AcurastStorage> {
        self.storages.get(&version).cloned().expect("Unsupported API version")
    }   
}

pub fn xxh128(data: &[u8], seed: u64) -> u128 {
    let mut buf = Cursor::new(data);
    let mut ret = Cursor::new(vec![0u8; 16]);

    let hash1 = xxh3_64_with_seed(buf.get_ref(), seed);
    buf.set_position(0);
    let hash2 = xxh3_64_with_seed(buf.get_ref(), seed + 1);
    
    ret.write_u64::<LittleEndian>(hash1).unwrap();
    ret.write_u64::<LittleEndian>(hash2).unwrap();

    ret.set_position(0);
    ret.read_u128::<LittleEndian>().unwrap()
}