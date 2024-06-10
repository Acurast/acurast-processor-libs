use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

#[async_trait]
pub trait RpcEngine: Send + Sync {
    async fn request(&self, body: Option<serde_json::Value>, timeout: Option<u64>) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
    fn id(&self) -> &str;
}

pub async fn rpc_engine_request(
    engine: &Arc<dyn RpcEngine>,
    method: &str,
    params: serde_json::Value,
    timeout: Option<u64>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let id = rand::random::<u64>().to_string();
    let body = json!({
        "id": id,
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    });
    engine.request(Some(body), timeout).await
}