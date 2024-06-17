use crate::engine::http::http_client::{HttpClient, HttpHeader};
use crate::engine::engine::RpcEngine;
use serde_json::{json, Value};
use std::time::Duration;
use async_trait::async_trait;
use std::sync::Arc;
pub struct HttpEngine {
    pub id: String,
    pub client: Arc<dyn HttpClient + Send + Sync>,
}

impl HttpEngine {
    pub fn new(id: String, client: Arc<dyn HttpClient + Send + Sync>) -> Self {
        HttpEngine { id, client }
    }
}

#[async_trait]
impl RpcEngine for HttpEngine {
    fn id(&self) ->  &str {
        todo!()
    }
    
    async fn request(
        &self,
        body: Option<serde_json::Value>,
        timeout: Option<u64>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let body_str = body.unwrap().to_string();   
        let content_type_header: HttpHeader = ("Content-Type".to_string(), Some("application/json".to_string()));
        let host_header: HttpHeader = ("Host".to_string(), Some("rust".to_string()));
        let headers = vec![content_type_header, host_header,];
        let response = self
            .client
            .post("https://canarynet-ws-1.acurast-h-server-2.papers.tech", Some(&body_str), Some(&headers), None, timeout, None)
            .await?;
        println!("HttpEngine::response: {}", response);
        let response_body = response;
        let json_response: serde_json::Value = serde_json::from_str(&response_body)?;
        Ok(json_response)
    }
}