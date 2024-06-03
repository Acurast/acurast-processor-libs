use async_trait::async_trait;
use reqwest::Error;
/// A key-value HTTP header.
pub type HttpHeader = (String, Option<String>);

/// A key-value HTTP parameter.
pub type HttpParameter = (String, Option<String>);

#[async_trait]
pub trait HttpClient {
    async fn delete(
        &self,
        base_url: &str,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error>;

    async fn get(
        &self,
        base_url: &str,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error>;

    async fn patch(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error>;

    async fn post(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error>;

    async fn put(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> Result<String, Error>;
}
