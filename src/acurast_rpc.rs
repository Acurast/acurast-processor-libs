use crate::codec::processor_update_info::ProcessorUpdateInfo;
use crate::codec::processor_version::ProcessorVersion;
use crate::engine::engine::RpcEngine;
use crate::engine::pallet::author::Author;
use crate::engine::pallet::chain::Chain;
use crate::engine::pallet::state::{RuntimeVersion, State};
use crate::hex::hex_to_ba;
use crate::storage::acurast_storage::StorageKey;
use crate::storage::v0_acurast_storage::V0AcurastStorageImpl;
use crate::storage::v0_acurast_storage::VXAcurastStorage;
use crate::storage::v1_acurast_storage::V1AcurastStorageImpl;
use crate::types::header::Header;
use async_trait::async_trait;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hex::decode;
use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64_with_seed;

#[derive(Debug, PartialEq, uniffi::Enum)]
pub enum AcurastRpcError {
    RuntimeError(String),
}

impl fmt::Display for AcurastRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AcurastRpcError::RuntimeError(msg) => write!(f, "Runtime Error: {}", msg),
        }
    }
}

#[uniffi::export]
#[async_trait]
pub trait AcurastRpc: Send + Sync {
    async fn get_block_hash(
        &self,
        block_number: Option<i64>,
        timeout: Option<u64>,
    ) -> Result<String, AcurastRpcError>;
    async fn get_header(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Header, AcurastRpcError>;
    async fn get_api_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> u32;
    async fn get_runtime_version(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<RuntimeVersion, AcurastRpcError>;
    async fn get_update_info(
        &self,
        account_id: Vec<u8>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<ProcessorUpdateInfo>;
    async fn get_known_binary_hash(
        &self,
        version: ProcessorVersion,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Vec<u8>>;
    async fn submit_extrinsic(
        &self,
        extrinsic: Vec<u8>,
        timeout: Option<u64>,
    ) -> Result<String, AcurastRpcError>;
    async fn call(
        &self,
        method: String,
        data: Option<Vec<u8>>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<String>;
    fn storage(&self, version: u32) -> Arc<dyn VXAcurastStorage>;
}

pub struct AcurastRpcImpl {
    engine: Arc<dyn RpcEngine>,
    state: Arc<State>,
    author: Arc<Author>,
    chain: Arc<Chain>,
    storages: HashMap<u32, Arc<dyn VXAcurastStorage>>,
}

impl AcurastRpcImpl {
    pub fn new(engine: Arc<dyn RpcEngine>) -> Self {
        let author = Arc::new(Author::new());
        let chain = Arc::new(Chain::new());
        let state = Arc::new(State::new());

        let storages: HashMap<u32, Arc<dyn VXAcurastStorage>> = vec![
            (
                0,
                Arc::new(V0AcurastStorageImpl::new(engine.clone(), state.clone(), 0))
                    as Arc<dyn VXAcurastStorage>,
            ),
            (
                1,
                Arc::new(V1AcurastStorageImpl::new(engine.clone(), state.clone()))
                    as Arc<dyn VXAcurastStorage>,
            ),
        ]
        .into_iter()
        .collect();

        Self {
            engine,
            state,
            author,
            chain,
            storages,
        }
    }
}

#[async_trait]
impl AcurastRpc for AcurastRpcImpl {
    async fn get_block_hash(
        &self,
        block_number: Option<i64>,
        timeout: Option<u64>,
    ) -> Result<String, AcurastRpcError> {
        self.chain
            .get_block_hash(block_number, timeout, &self.engine)
            .await
            .map_err(|e| AcurastRpcError::RuntimeError(format!("{}", e)))
    }

    async fn get_header(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Header, AcurastRpcError> {
        self.chain
            .get_header(block_hash, timeout, &self.engine)
            .await
            .map_err(|e| AcurastRpcError::RuntimeError(format!("{}", e)))
    }

    async fn get_api_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> u32 {
        let key = acurast_processor_manager_api_version();
        match self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            Ok(storage_option) => {
                log::debug!("get_api_version {:?}", storage_option);
                if let Some(storage) = storage_option {
                    let decoded_storage = hex_to_ba(&storage);
                    let mut cursor = std::io::Cursor::new(decoded_storage);
                    cursor.read_u32::<LittleEndian>().unwrap_or(0)
                } else {
                    0
                }
            }
            Err(_) => 0,
        }
    }

    async fn get_runtime_version(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<RuntimeVersion, AcurastRpcError> {
        self.state
            .get_runtime_version(block_hash, timeout, &self.engine)
            .await
            .map_err(|e| AcurastRpcError::RuntimeError(format!("{}", e)))
    }

    async fn get_update_info(
        &self,
        account_id: Vec<u8>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<ProcessorUpdateInfo> {
        let key = acurast_processor_manager_processor_update_info(&account_id);
        match self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            Ok(storage_option) => {
                if let Some(storage) = storage_option {
                    let byte_array = hex_to_ba(&storage);
                    let mut cursor = Cursor::new(&byte_array[..]);
                    Some(ProcessorUpdateInfo::read(&mut cursor))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    async fn get_known_binary_hash(
        &self,
        version: ProcessorVersion,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<Vec<u8>> {
        let key = acurast_processor_manager_known_binary_hash(&version);
        match self
            .state
            .get_storage(&key, block_hash, timeout, &self.engine)
            .await
        {
            Ok(storage_option) => {
                if let Some(storage) = storage_option {
                    decode(storage).ok()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    async fn submit_extrinsic(
        &self,
        extrinsic: Vec<u8>,
        timeout: Option<u64>,
    ) -> Result<String, AcurastRpcError> {
        self.author
            .submit_extrinsic(extrinsic, timeout, &self.engine)
            .await
            .map_err(|e| AcurastRpcError::RuntimeError(e))
    }

    async fn call(
        &self,
        method: String,
        data: Option<Vec<u8>>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Option<String> {
        self.call(method, data, block_hash, timeout).await
    }

    fn storage(&self, version: u32) -> Arc<dyn VXAcurastStorage> {
        self.storages
            .get(&version)
            .cloned()
            .expect("Unsupported API version")
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

fn acurast_processor_manager_api_version() -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix("AcurastProcessorManager", "ApiVersion");

    key
}

fn acurast_processor_manager_processor_update_info(account_id: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_with_storage_prefix("AcurastProcessorManager", "ProcessorUpdateInfo");
    key.extend_with_blake2_128concat(account_id);

    key
}

fn acurast_processor_manager_known_binary_hash(version: &ProcessorVersion) -> Vec<u8> {
    let version = version.to_u8a();

    let mut key = Vec::new();
    key.extend_with_storage_prefix("AcurastProcessorManager", "KnownBinaryHash");
    key.extend_with_blake2_128concat(&version);

    key
}

#[cfg(test)]
mod tests {
    use crate::Platform;

    use super::*;

    #[test]
    fn test_acurast_processor_manager_api_version() {
        let key = acurast_processor_manager_api_version();

        assert_eq!(
            key,
            hex_to_ba("0xc698b31ad72d251a494de23e0dd66846b0ec0523d482e30628c3691600bb72ae")
        );
    }

    #[test]
    fn test_acurast_processor_manager_processor_update_info() {
        let key = acurast_processor_manager_processor_update_info(&hex_to_ba(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ));

        assert_eq!(
            key,
            hex_to_ba("0xc698b31ad72d251a494de23e0dd6684688192b208e46ab5e9fa0efa03baeffd9de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
        );
    }

    #[test]
    fn test_acurastp_pocessor_manager_known_binary_hash() {
        let key = acurast_processor_manager_known_binary_hash(&ProcessorVersion {
            platform: Platform::Android,
            build_number: 0,
        });

        assert_eq!(
            key,
            hex_to_ba("0xc698b31ad72d251a494de23e0dd66846e8e28ace40923c7aded0457baf4aba6ec804ce198ec337e3dc762bdd1a09aece0000000000000000"),
        );
    }
}
