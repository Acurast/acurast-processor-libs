use async_trait::async_trait;
use std::collections::HashMap;

/// A key-value HTTP header.
pub type HttpHeader = (String, Option<String>);

/// A key-value HTTP parameter.
pub type HttpParameter = (String, Option<String>);

/// HTTP client provider trait.
///
/// Use this trait to register a custom HTTP client implementation.
#[async_trait]
pub trait HttpClient {
    /// Call DELETE HTTP method on the specified `base_url` with `headers` and `parameters`.
    ///
    /// The method returns a JSON response encoded as `String`.
    async fn delete(
        &self,
        base_url: &str,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> String;

    /// Call GET HTTP method on the specified `base_url` with `headers` and `parameters`.
    ///
    /// The method returns a JSON response encoded as `String`.
    async fn get(
        &self,
        base_url: &str,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> String;

    /// Call PATCH HTTP method on the specified `base_url` with `headers`, `parameters`, and `body`.
    ///
    /// The `body` is a serialized JSON in `String` representation.
    /// The method returns a JSON response encoded as `String`.
    async fn patch(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> String;

    /// Call POST HTTP method on the specified `base_url` with `headers`, `parameters`, and `body`.
    ///
    /// The `body` is a serialized JSON in `String` representation.
    /// The method returns a JSON response encoded as `String`.
    async fn post(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> String;

    /// Call PUT HTTP method on the specified `base_url` with `headers`, `parameters`, and `body`.
    ///
    /// The `body` is a serialized JSON in `String` representation.
    /// The method returns a JSON response encoded as `String`.
    async fn put(
        &self,
        base_url: &str,
        body: Option<&str>,
        headers: Option<&[HttpHeader]>,
        parameters: Option<&[HttpParameter]>,
        request_timeout: Option<u64>,
        connection_timeout: Option<u64>,
    ) -> String;
}
