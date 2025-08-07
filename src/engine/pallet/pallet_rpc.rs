use async_trait::async_trait;
use serde_json::Value;

const UNKNOWN_ERROR: &str = "Unknown Error";

#[async_trait]
pub trait PalletRpc {
    fn read_string_result(&self, response: Result<serde_json::Value, Box<dyn std::error::Error>>) -> Result<Option<String>, String> {
        self.read_result(response, |json| self.get_opt_string_result(json))
    }

    fn read_result<U, F>(&self, response: Result<serde_json::Value, Box<dyn std::error::Error>>, get_result: F) -> Result<U, String> 
    where
        F: FnOnce(&Value) -> Result<Option<U>, String>,
    {
        match response {
            Ok(response) => {
                if let Some(result) = get_result(&response)? {
                    Ok(result)
                } else {
                    Err(self.get_error(&response))
                }
            }
            Err(err) => {
                Err(err.to_string())
            }
        }
    }

    fn get_opt_string_result(&self, json: &Value) -> Result<Option<Option<String>>, String> {
        self.get_opt_result(json, |json| {
            if let Some(str) = json.as_str() {
                Ok(str.to_string())
            } else {
                Err("Unexpected result type (!= String)".to_string())
            }
        })
    }

    fn get_opt_result<U, F>(&self, json: &Value, transform: F) -> Result<Option<Option<U>>, String>
    where
        F: FnOnce(&Value) -> Result<U, String>,
    {
        let result = json.get("result");

        if let Some(_) = result.and_then(Value::as_null) {
            Ok(Some(None))
        } else if let Some(result) = result {
            let result = transform(result)?;
            Ok(Some(Some(result)))
        } else {
            Ok(None)
        }
    }

    fn get_error(&self, json: &Value) -> String {
        if let Some(error) = json.get("error") {
            let message = error["message"].as_str().unwrap_or(UNKNOWN_ERROR);
            let data = error["data"].as_str().unwrap_or_default();
            let error = if !data.is_empty() {
                format!("{} ({})", message, data)
            } else {
                message.to_string()
            };

            error
        } else {
            UNKNOWN_ERROR.to_string()
        }
    }
}
