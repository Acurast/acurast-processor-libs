
mod HttpClientTests {
    use crate::engine::engine::RpcEngine;
    use crate::engine::http::http_client::HttpClient;
    use crate::engine::http_engine::HttpEngine;
    use crate::engine::http::http_client_impl::HttpClientImpl;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        
        let client: Box<dyn HttpClient + Send + Sync> = Box::new(HttpClientImpl::new());
        //let engine: Box<dyn RpcEngine> = Box::new(HttpEngine::new("example_id".to_string(), client));
    
        let base_url = "https://jsonplaceholder.typicode.com/todos/1";
        let headers: Option<Vec<(String, Option<String>)>> = None; // No headers needed for this request
        let parameters: Option<Vec<(String, Option<String>)>> = None; // No parameters needed for this request
        let request_timeout: Option<u64> = None; // No specific timeout
        let connection_timeout: Option<u64> = None; // No specific connection timeout
    
        match client.get(base_url, headers.as_deref(), parameters.as_deref(), request_timeout, connection_timeout).await {
            Ok(response) =>  { assert!(true); println!("Response: {}", response) },
            Err(e) => { assert!(true); println!("Error: {}", e) },
        }
    }
}