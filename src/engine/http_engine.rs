use crate::engine::engine::RpcEngine;
use crate::engine::http::http_client::{HttpClient, HttpHeader};
use async_trait::async_trait;
use std::sync::Arc;

pub struct HttpRpcEngine {
    pub url: String,
    pub client: Arc<dyn HttpClient + Send + Sync>,
}

impl HttpRpcEngine {
    pub fn new(url: String, client: Arc<dyn HttpClient + Send + Sync>) -> Self {
        HttpRpcEngine { url, client }
    }
}

#[async_trait]
impl RpcEngine for HttpRpcEngine {
    fn id(&self) -> &str {
        todo!()
    }

    async fn request(
        &self,
        body: Option<serde_json::Value>,
        timeout: Option<u64>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let body_str = body.unwrap().to_string();
        let content_type_header: HttpHeader = (
            "Content-Type".to_string(),
            Some("application/json".to_string()),
        );
        let headers = vec![content_type_header];
        let response = self
            .client
            .post(
                self.url.as_str(),
                Some(&body_str),
                Some(&headers),
                None,
                timeout,
                None,
            )
            .await?;
        log::debug!("HttpEngine::response: {}", response);
        let response_body = response;
        let json_response: serde_json::Value = serde_json::from_str(&response_body)?;
        Ok(json_response)
    }
}
