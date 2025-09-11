uniffi::setup_scaffolding!();

pub mod acurast_rpc;
pub mod big_uint;
pub mod blake2b;
pub mod codec;
pub mod cursor;
pub mod engine;
mod error;
pub mod hex;
pub mod storage;
pub mod tests;
pub mod types;
pub mod u8a;
pub mod uint128;
pub mod vec_slice;
pub mod xxhash;

pub use crate::codec::platform::Platform;
pub use crate::codec::processor_update_info::ProcessorUpdateInfo;
pub use crate::codec::processor_version::ProcessorVersion;
use crate::engine::http::http_client::HttpClient;
use crate::engine::http::http_client_impl::HttpClientImpl;
pub use crate::engine::pallet::state::RuntimeVersion;
pub use crate::error::AcurastError;
use crate::storage::acurast_storage::AcurastStorage;
pub use crate::types::header::Header;
pub use acurast_rpc::AcurastRpc;
pub use acurast_rpc::AcurastRpcImpl;
use engine::engine::RpcEngine;
use engine::http_engine::HttpRpcEngine;
use num_bigint::BigInt;
use num_bigint::BigUint;
use num_rational::BigRational;
use std::str::FromStr;
use std::sync::Arc;

pub type Result<T> = core::result::Result<T, AcurastError>;

fn setup_logger() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Stdout)
        .init();
}

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

#[derive(uniffi::Record, Debug)]
pub struct SerBigRational {
    pub numer: String,
    pub denom: String,
}

uniffi::custom_type!(BigRational, SerBigRational);
impl UniffiCustomTypeConverter for BigRational {
    type Builtin = SerBigRational;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        let numer = BigInt::from_str(&val.numer)?;
        let denom = BigInt::from_str(&val.denom)?;

        Ok(Self::new(numer, denom))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        Self::Builtin {
            numer: obj.numer().to_string(),
            denom: obj.denom().to_string(),
        }
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
pub struct AcurastRpcClient {
    pub acurastrpc: Arc<dyn AcurastRpc>,
}

#[uniffi::export]
impl AcurastRpcClient {
    #[uniffi::constructor]
    pub fn new(acurast_rpc: Arc<dyn AcurastRpc>) -> Arc<Self> {
        setup_logger();
        Arc::new(Self {
            acurastrpc: acurast_rpc,
        })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl AcurastRpcClient {
    pub async fn get_header(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Header> {
        self.acurastrpc.get_header(block_hash, timeout).await
    }
    pub async fn call(
        &self,
        method: String,
        data: Option<Vec<u8>>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<Option<String>> {
        self.acurastrpc
            .call(method, data, block_hash, timeout)
            .await
    }
    pub async fn get_runtime_version(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> Result<RuntimeVersion> {
        self.acurastrpc
            .get_runtime_version(block_hash, timeout)
            .await
    }
    pub async fn get_block_hash(
        &self,
        block_number: Option<i64>,
        timeout: Option<u64>,
    ) -> Result<Option<String>> {
        self.acurastrpc.get_block_hash(block_number, timeout).await
    }
    pub async fn submit_extrinsic(
        &self,
        extrinsic: Vec<u8>,
        timeout: Option<u64>,
    ) -> Result<Option<String>> {
        self.acurastrpc.submit_extrinsic(extrinsic, timeout).await
    }
    pub fn storage(&self, version: u32) -> Arc<AcurastStorage> {
        let storage = self.acurastrpc.storage(version);
        AcurastStorage::new(storage)
    }
    pub async fn get_storage_version(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
    ) -> u32 {
        self.acurastrpc.get_api_version(block_hash, timeout).await
    }
}
