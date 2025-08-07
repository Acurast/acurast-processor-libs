use crate::engine::engine::{rpc_engine_request, RpcEngine};
use crate::engine::pallet::pallet_rpc::PalletRpc;
use crate::hex::ToHex;
use serde_json::json;
use std::sync::Arc;

pub struct Author {}

impl PalletRpc for Author {}

impl Author {
    pub fn new() -> Self {
        Author {}
    }

    pub async fn submit_extrinsic(
        &self,
        extrinsic: Vec<u8>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Option<String>, String> {
        let params = json!([extrinsic.to_hex()]);
        let response = rpc_engine_request(engine, "author_submitExtrinsic", params, timeout).await;

        self.read_string_result(response)
    }
}
