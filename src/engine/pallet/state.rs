use crate::engine::engine::{rpc_engine_request, RpcEngine};
use std::sync::Arc;
use serde_json::{json, Value};

pub struct RuntimeVersion {
    spec_version: i32,
    transaction_version: i32,
}

impl RuntimeVersion {
    pub fn new(spec_version: i32, transaction_version: i32) -> Self {
        Self {
            spec_version,
            transaction_version,
        }
    }

    // Getters
    pub fn spec_version(&self) -> i32 {
        self.spec_version
    }

    pub fn transaction_version(&self) -> i32 {
        self.transaction_version
    }
}


pub struct State {

}

impl State {
    pub fn new() -> Self {
        State {  }
    }

    pub async fn get_runtime_version(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<RuntimeVersion, Box<dyn std::error::Error>> {
        let params = json!([block_hash.map(|b| hex::encode(b))]);
        let response = rpc_engine_request(engine, "state_getRuntimeVersion", params, timeout).await?;

        let result = response
            .get("result")
            .and_then(|r| r.as_object())
            .ok_or("Invalid response")?;

        let spec_version = result.get("specVersion").and_then(|v| v.as_i64()).map(|v| v as i32);
        let transaction_version = result.get("transactionVersion").and_then(|v| v.as_i64()).map(|v| v as i32);
        
        Ok(RuntimeVersion::new(spec_version.unwrap(), transaction_version.unwrap()))
    }

    pub async fn get_storage(
        &self,
        storage_key: &[u8],
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut params = json!([storage_key]);
    
        if let Some(block_hash) = block_hash {
            params.as_array_mut().unwrap().push(Value::from(block_hash));
        }
        let response = rpc_engine_request(engine, "state_getStorage", params, timeout).await?;
        let result = response.get("result").and_then(Value::as_str);
        Ok(result.map(String::from))
    }
}