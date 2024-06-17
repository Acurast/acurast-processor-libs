pub mod engine;
pub mod acurast_rpc;
pub mod storage;
pub mod codec;
pub mod types;
pub mod tests;
pub mod hex;

uniffi::include_scaffolding!("acurastsdk");
use std::str::FromStr;
use std::sync::Arc;
pub use acurast_rpc::AcurastRpc;
pub use acurast_rpc::AcurastRpcImpl;
use engine::engine::RpcEngine;
use engine::http_engine::HttpEngine;
pub use crate::types::header::Header;
pub use crate::engine::pallet::state::RuntimeVersion;
pub use crate::codec::processor_update_info::ProcessorUpdateInfo;
pub use crate::codec::processor_version::ProcessorVersion;
pub use crate::codec::platform::Platform;
use crate::storage::acurast_storage::AcurastStorage;
use crate::engine::http::http_client::HttpClient;
use crate::engine::http::http_client_impl::HttpClientImpl;
use num_bigint::BigUint;


impl UniffiCustomTypeConverter for BigUint {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(BigUint::from_str(&val).unwrap())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

pub fn acurast_rpc() -> Arc<dyn AcurastRpc> {
    let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(HttpClientImpl::new());
    let engine: Arc<dyn RpcEngine> = Arc::new(HttpEngine::new("example_id".to_string(), client.clone()));
    let rpc: AcurastRpcImpl = AcurastRpcImpl::new(engine.clone());
    Arc::new(rpc)
}