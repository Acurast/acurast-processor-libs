pub mod engine;

#[cfg(test)]
mod tests {
    use engine::{engine::RpcEngine, http::http_client::HttpClient};
    use engine::http_engine::HttpEngine;
    use engine::http::http_client_impl::HttpClientImpl;

    use super::*;

    #[test]
    fn it_works() {
        let client: Box<dyn HttpClient + Send + Sync> = Box::new(HttpClientImpl::new());
    let engine: Box<dyn RpcEngine> = Box::new(HttpEngine::new("example_id".to_string(), client));
        //let engine: &dyn RpcEngine = HttpEngine { client: HttpClient{} };
        //assert_eq!(result, 4);
    }
}
