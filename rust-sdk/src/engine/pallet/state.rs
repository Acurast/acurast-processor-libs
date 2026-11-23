use crate::engine::pallet::pallet_rpc::PalletRpc;
use crate::hex::ToHex;
use crate::{
    engine::engine::{rpc_engine_request, RpcEngine},
    types::storage::StorageQueryResult,
};
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct RuntimeVersion {
    pub spec_version: i32,
    pub transaction_version: i32,
}

impl RuntimeVersion {
    pub fn new(spec_version: i32, transaction_version: i32) -> Self {
        Self {
            spec_version,
            transaction_version,
        }
    }
}

pub struct State;

impl PalletRpc for State {}

impl State {
    pub fn new() -> Self {
        State {}
    }

    pub async fn get_runtime_version(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<RuntimeVersion, Box<dyn Error>> {
        let params = json!([block_hash.map(|b| hex::encode(b))]);
        let response =
            rpc_engine_request(engine, "state_getRuntimeVersion", params, timeout).await?;

        let result = response
            .get("result")
            .and_then(|r| r.as_object())
            .ok_or("Invalid response")?;

        let spec_version = result
            .get("specVersion")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        let transaction_version = result
            .get("transactionVersion")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        Ok(RuntimeVersion::new(
            spec_version.unwrap(),
            transaction_version.unwrap(),
        ))
    }

    pub async fn get_storage(
        &self,
        storage_key: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Option<String>, String> {
        let mut params = json!([hex::encode(storage_key)]);

        if let Some(block_hash) = block_hash {
            params.as_array_mut().unwrap().push(Value::from(block_hash));
        }
        let response = rpc_engine_request(engine, "state_getStorage", params, timeout).await;

        self.read_string_result(response)
    }

    pub async fn get_storage_at(
        &self,
        storage_keys: Vec<String>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Vec<StorageQueryResult>, String> {
        let mut params = json!([storage_keys]);
        if let Some(block_hash) = block_hash {
            params.as_array_mut().unwrap().push(Value::from(block_hash));
        }
        let response = rpc_engine_request(engine, "state_queryStorageAt", params, timeout).await;

        self.read_result(response, |json| {
            self.get_opt_result(json, |result| {
                let results: Vec<StorageQueryResult> = serde_json::from_value(result.clone())
                    .map_err(|e| format!("Failed to parse result: {}", e))?;

                Ok(results)
            })
        }).map(|ok| ok.unwrap_or(vec![]))
    }

    pub async fn call_state(
        &self,
        method: String,
        data: Option<Vec<u8>>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Option<String>, String> {
        let mut params = json!([method]);
        if let Some(data) = data {
            params.as_array_mut().unwrap().push(json!(data.to_hex()));
        }
        if let Some(block_hash) = block_hash {
            params
                .as_array_mut()
                .unwrap()
                .push(json!(block_hash.to_hex()));
        }
        let response = rpc_engine_request(&engine, "state_call", params, timeout).await;

        self.read_string_result(response)
    }

    pub async fn get_keys(
        &self,
        key: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Vec<String>, String> {
        let mut params = json!([hex::encode(key)]);
        if let Some(block_hash) = block_hash {
            params.as_array_mut().unwrap().push(Value::from(block_hash));
        }
        let response = rpc_engine_request(&engine, "state_getKeys", params, timeout).await;

        self.read_result(response, |json| {
            self.get_opt_result(json, |result| {
                if let Some(array) = result.as_array() {
                    let keys: Vec<String> = array
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();

                    Ok(keys)
                } else {
                    Err("Unexpected result type (!= Vec<String>)".to_string())
                }
            })
        }).map(|ok| ok.unwrap_or(vec![]))
    }
}
