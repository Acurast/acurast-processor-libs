use async_trait::async_trait;
use blake2::{Blake2b, Digest};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use hex::decode;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use xxhash_rust::xxh3::{xxh3_64, xxh3_64_with_seed};
use crate::storage::acurast_storage::AcurastStorage;
use crate::codec::processor_version::ProcessorVersion;
use std::io::{Cursor, Read, Write};

use crate::engine::engine::RpcEngine;
use crate::engine::pallet::author::Author;
use crate::engine::pallet::chain::Chain;
use crate::engine::pallet::state::State;
use crate::types::header::Header;

#[async_trait]
pub trait AcurastRpc {
    async fn get_block_hash(&self, block_number: Option<u64>, timeout: Option<u64>) -> Option<String>;
    async fn get_header(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Header;
    async fn get_api_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> u32;
    async fn get_runtime_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> RuntimeVersion;
    async fn get_update_info(&self, account_id: Vec<u8>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<ProcessorUpdateInfo>;
    async fn get_known_binary_hash(&self, version: ProcessorVersion, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<Vec<u8>>;
    async fn submit_extrinsic(&self, extrinsic: Vec<u8>, timeout: Option<u64>) -> Option<String>;
    async fn call(&self, method: String, data: Option<Vec<u8>>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<String>;
    fn storage(&self, version: u32) -> Arc<dyn AcurastStorage>;
}

pub struct AcurastRpcImpl {
    engine: Arc<dyn RpcEngine>,
    state: Arc<State>,
    author: Arc<Author>,
    chain: Arc<Chain>,
    storages: HashMap<u32, Arc<dyn AcurastStorage>>,
}

impl AcurastRpcImpl {
    pub fn new(engine: Arc<dyn RpcEngine>) -> Self {
        let author = Arc::new(Author::new());
        let chain = Arc::new(Chain::new());
        let state = Arc::new(State::new());

        let storages: HashMap<u32, Arc<dyn AcurastStorage>> = vec![
            //(0, Arc::new(V0AcurastStorage::new(engine.clone(), state.clone())) as Arc<dyn AcurastStorage>),
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

    fn hash_blake2b(data: &[u8]) -> Vec<u8> {
        let mut hasher = Blake2b::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

#[async_trait]
impl AcurastRpc for AcurastRpcImpl {
    async fn get_block_hash(&self, block_number: Option<u64>, timeout: Option<u64>) -> Option<String> {
        self.chain.get_block_hash(block_number, timeout, &self.engine).await
    }

    async fn get_header(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Header {
        self.chain.get_header(block_hash, timeout, &self.engine).await
    }

    async fn get_api_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> u32 {
        let key = Self::generate_key("AcurastProcessorManager", "ApiVersion");
        if let Some(storage) = self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            let decoded_storage = decode(&storage).expect("Decoding failed");
            let mut cursor = std::io::Cursor::new(decoded_storage);
            cursor.read_u32::<BigEndian>().unwrap_or(0)
        } else {
            0
        }
    }

    async fn get_runtime_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> RuntimeVersion {
        self.state.get_runtime_version(block_hash, timeout, &self.engine).await
    }

    async fn get_update_info(&self, account_id: Vec<u8>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<ProcessorUpdateInfo> {
        let key = [
            Self::generate_key("AcurastProcessorManager", "ProcessorUpdateInfo"),
            Self::hash_blake2b(&account_id),
            account_id.clone(),
        ].concat();
        if let Some(storage) = self.state.get_storage(&key, block_hash, timeout, &self.engine).await {
            Some(ProcessorUpdateInfo::read(&storage))
        } else {
            None
        }
    }

    async fn get_known_binary_hash(&self, version: ProcessorVersion, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<Vec<u8>> {
        let version_bytes = version.to_u8a();
        let key = [
            Self::generate_key("AcurastProcessorManager", "KnownBinaryHash"),
            Self::hash_blake2b(&version_bytes),
            version_bytes.clone(),
        ].concat();
        self.state.get_storage(&key, block_hash, timeout, &self.engine).await.map(|storage| decode(&storage).expect("Decoding failed"))
    }

    async fn submit_extrinsic(&self, extrinsic: Vec<u8>, timeout: Option<u64>) -> Option<String> {
        self.author.submit_extrinsic(extrinsic, timeout, &self.engine).await
    }

    async fn call(&self, method: String, data: Option<Vec<u8>>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<String> {
        self.state.call(method, data, block_hash, timeout, &self.engine).await
    }

    fn storage(&self, version: u32) -> Arc<dyn AcurastStorage> {
        self.storages.get(&version).cloned().expect("Unsupported API version")
    }
    
}


fn xxh128(data: &[u8], seed: u64) -> u128 {
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