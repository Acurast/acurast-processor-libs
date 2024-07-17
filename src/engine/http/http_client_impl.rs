use reqwest::Body;
use reqwest::Client;
use reqwest::Error;
use std::collections::HashMap;
use crate::engine::http::http_client::HttpClient;
use crate::engine::http::http_client::HttpHeader;
use crate::engine::http::http_client::HttpParameter;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub struct HttpClientImpl {
    client: Client,
}

impl HttpClientImpl {
    pub fn new() -> Self {
        HttpClientImpl { client: Client::new() }
    }

    fn build_headers(&self, headers: Option<&[HttpHeader]>) -> HeaderMap {
        let mut header_map = HeaderMap::new();
        if let Some(headers) = headers {
            for (key, value) in headers {
                if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                    if let Some(value) = value {
                        if let Ok(header_value) = HeaderValue::from_str(&value) {
                            header_map.insert(header_name, header_value);
                        }
                    }
                }
            }
        }
        header_map
    }

    fn build_params(&self, params: Option<&[HttpParameter]>) -> HashMap<String, String> {
        let mut param_map = HashMap::new();
        if let Some(params) = params {
            for (key, value) in params {
                if let Some(value) = value {
                    param_map.insert(key.clone(), value.clone());
                }
            }
        }
        param_map
    }
}

#[async_trait]
impl HttpClient for HttpClientImpl {
    async fn delete(
        &self,
        base_url: &str,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error> {
        let response = self.client
            .delete(base_url)
            .headers(self.build_headers(headers))
            .query(&self.build_params(parameters))
            .send()
            .await?
            .text()
            .await;
        response
    }

    async fn get(
        &self,
        base_url: &str,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error> {
        let response = self.client
            .get(base_url)
            .headers(self.build_headers(headers))
            .query(&self.build_params(parameters))
            .send()
            .await?
            .text()
            .await;
        response
    }

    async fn patch(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error> {
        let response = self.client
            .patch(base_url)
            .headers(self.build_headers(headers))
            .query(&self.build_params(parameters))
            .body(Body::from(body.unwrap_or("").to_string()))
            .send()
            .await?
            .text()
            .await;
        response
    }

    async fn post(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error> {
        println!("HttpClientImpl::post: {}", base_url);
        let response = self.client
            .post(base_url)
            .headers(self.build_headers(headers))
            .query(&self.build_params(parameters))
            .body(Body::from(body.unwrap_or("").to_string()))
            .send()
            .await?
            .text()
            .await;
        response
    }

    async fn put(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error> {
        let response = self.client
            .put(base_url)
            .headers(self.build_headers(headers))
            .query(&self.build_params(parameters))
            .body(Body::from(body.unwrap_or("").to_string()))
            .send()
            .await?
            .text()
            .await;
        response
    }
}