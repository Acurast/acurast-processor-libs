use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;

#[async_trait]
pub trait RpcEngine {
    fn id(&self) -> &str;

    async fn request(&self, body: Value, timeout: Option<Duration>) -> Value;
}

pub async fn request(
    engine: &dyn RpcEngine,
    id: u64,
    method: &str,
    params: Vec<Value>,
    timeout: Option<Duration>,
) -> Value {
    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    });

    engine.request(body, timeout).await
}