pub mod engine;
pub mod acurast_rpc;
pub mod storage;
pub mod codec;
pub mod types;
pub mod tests;
pub mod hex;
pub mod xxhash;
pub mod vec_slice;
pub mod u8a;

use async_trait::async_trait;
use storage::v0_acurast_storage::V0AcurastStorageImpl;
uniffi::setup_scaffolding!();
use std::str::FromStr;
use std::sync::Arc;
pub use acurast_rpc::AcurastRpc;
pub use acurast_rpc::AcurastRpcImpl;
use engine::engine::RpcEngine;
use engine::http_engine::HttpRpcEngine;
pub use crate::types::header::Header;
pub use crate::engine::pallet::state::RuntimeVersion;
pub use crate::codec::processor_update_info::ProcessorUpdateInfo;
pub use crate::codec::processor_version::ProcessorVersion;
pub use crate::codec::platform::Platform;
use crate::engine::http::http_client::HttpClient;
use crate::engine::http::http_client_impl::HttpClientImpl;
use crate::storage::v0_acurast_storage::V0AcurastStorage;
use crate::storage::acurast_storage::AcurastStorage;
use num_bigint::BigUint;


uniffi::custom_type!(BigUint, String);
impl UniffiCustomTypeConverter for BigUint {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(BigUint::from_str(&val).unwrap())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

#[uniffi::export]
pub fn acurast_rpc(url: String) -> Arc<dyn AcurastRpc> {
    let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(HttpClientImpl::new());
    let engine: Arc<dyn RpcEngine> = Arc::new(HttpRpcEngine::new(url, client.clone()));
    let rpc: AcurastRpcImpl = AcurastRpcImpl::new(engine.clone());
    Arc::new(rpc)
}

#[derive(uniffi::Object)]
pub struct AcurastRpcClient{ 
    pub acurastrpc: Arc<dyn AcurastRpc>,
}

#[uniffi::export]
impl AcurastRpcClient {
    #[uniffi::constructor]
    pub fn new(acurast_rpc: Arc<dyn AcurastRpc>) -> Arc<Self> {
        Arc::new(Self { acurastrpc: acurast_rpc })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl AcurastRpcClient {
    pub async fn get_header(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<Header> {
        self.acurastrpc.get_header(block_hash, timeout).await
    }
    pub async fn call(&self, method: String, data: Option<Vec<u8>>, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> Option<String>{
        self.acurastrpc.call(method, data, block_hash, timeout).await
    }
    pub async fn get_runtime_version(&self, block_hash: Option<Vec<u8>>, timeout: Option<u64>) -> RuntimeVersion {
        self.acurastrpc.get_runtime_version(block_hash, timeout).await
    }
    pub async fn get_block_hash(&self, block_number: Option<i64>, timeout: Option<u64>) -> Option<String> {
        self.acurastrpc.get_block_hash(block_number, timeout).await
    }
    pub fn storage(&self, version: u32) -> Arc<AcurastStorage> {
        let storage = self.acurastrpc.storage(version);
        AcurastStorage::new(storage)
    }
}