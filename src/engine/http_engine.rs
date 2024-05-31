use crate::engine::http::http_client::HttpClient;
use crate::engine::engine::RpcEngine;
use serde_json::{json, Value};
use std::time::Duration;
pub struct HttpEngine {
    pub id: String,
    pub client: Box<dyn HttpClient + Send + Sync>,
}

impl HttpEngine {
    pub fn new(id: String, client: Box<dyn HttpClient + Send + Sync>) -> Self {
        HttpEngine { id, client }
    }
}

impl RpcEngine for HttpEngine {
    fn id(&self) ->  &str {
        todo!()
    }

    #[must_use]
#[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
fn request<'life0,'async_trait>(&'life0 self,body:Value,timeout:Option<Duration>) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Value> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }
}