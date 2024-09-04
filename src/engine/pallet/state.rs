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
    ) -> Result<Option<String>, Box<dyn Error>> {
        let mut params = json!([hex::encode(storage_key)]);

        if let Some(block_hash) = block_hash {
            params.as_array_mut().unwrap().push(Value::from(block_hash));
        }
        let response = rpc_engine_request(engine, "state_getStorage", params, timeout).await?;
        let result = response.get("result").and_then(Value::as_str);
        Ok(result.map(String::from))
    }

    pub async fn get_storage_at(
        &self,
        storage_keys: Vec<String>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Vec<StorageQueryResult>, Box<dyn Error>> {
        let mut params = json!([storage_keys]);
        if let Some(block_hash) = block_hash {
            params.as_array_mut().unwrap().push(Value::from(block_hash));
        }
        let response = rpc_engine_request(engine, "state_queryStorageAt", params, timeout).await?;
        match response.get("result") {
            Some(result) => {
                // Convert the JSON result to a vector of StorageQueryResult
                let results: Vec<StorageQueryResult> = serde_json::from_value(result.clone())
                    .map_err(|e| format!("Failed to parse result: {}", e))?;
                Ok(results)
            }
            None => Err("Response does not contain result key".into()),
        }
    }

    pub async fn call_state(
        method: String,
        data: Option<Vec<u8>>,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: Arc<dyn RpcEngine>,
    ) -> Result<String, String> {
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
        match rpc_engine_request(&engine, "state_call", params, timeout).await {
            Ok(response) => {
                if let Some(result) = response.get("result").and_then(Value::as_str) {
                    Ok(result.to_string())
                } else {
                    Err("Result is not a string".to_string())
                }
            }
            Err(err) => {
                let err_msg = format!("Error: {}", err);
                Err(err_msg)
            }
        }
    }

    pub async fn get_keys(
        &self,
        key: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut params = json!([hex::encode(key)]);
        if let Some(block_hash) = block_hash {
            params.as_array_mut().unwrap().push(Value::from(block_hash));
        }
        match rpc_engine_request(&engine, "state_getKeys", params, timeout).await {
            Ok(response) => match response["result"].as_array() {
                Some(array) => Ok(array
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()),
                None => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to retrieve keys",
                ))),
            },
            Err(_err) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to retrieve keys",
            ))),
        }
    }
}
