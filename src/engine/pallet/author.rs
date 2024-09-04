use crate::engine::engine::{rpc_engine_request, RpcEngine};
use crate::hex::ToHex;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct Author {}

impl Author {
    pub fn new() -> Self {
        Author {}
    }

    pub async fn submit_extrinsic(
        &self,
        extrinsic: Vec<u8>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<String, String> {
        let params = json!([extrinsic.to_hex()]);
        match rpc_engine_request(engine, "author_submitExtrinsic", params, timeout).await {
            Ok(response) => {
                if let Some(result) = response.get("result").and_then(Value::as_str) {
                    Ok(result.to_string())
                } else {
                    Err("Result is not a string".to_string())
                }
            }
            Err(err) => {
                let err_msg = format!("Error: {}", err); // Convert the error to a String
                Err(err_msg)
            }
        }
    }
}
