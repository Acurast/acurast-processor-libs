use serde_json::{json, Value};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
trait PalletRpc {
    fn handle_error(json: Value) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(error) = json.get("error") {
            let message = error["message"].as_str().unwrap_or_default();
            let data = error["data"].as_str().unwrap_or_default();
            let exception_message = if !data.is_empty() {
                format!("{} ({})", message, data)
            } else {
                message.to_string()
            };
            return Err(exception_message.into());
        }
        Ok(None)
    }
}