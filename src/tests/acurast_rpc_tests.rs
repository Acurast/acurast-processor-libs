

#[cfg(test)]
pub mod acurastTests{
    use crate::engine::engine::RpcEngine;
    use crate::engine::http::http_client::HttpClient;
    use crate::engine::http_engine::HttpEngine;
    use crate::engine::http::http_client_impl::HttpClientImpl;
    use crate::acurast_rpc::{AcurastRpc, AcurastRpcImpl};
    use std::sync::Arc;

    lazy_static::lazy_static! {
        static ref CLIENT: Arc<dyn HttpClient + Send + Sync> = Arc::new(HttpClientImpl::new());
        static ref ENGINE: Arc<dyn RpcEngine> = Arc::new(HttpEngine::new("example_id".to_string(), CLIENT.clone()));
        static ref AC_RPC: AcurastRpcImpl = AcurastRpcImpl::new(ENGINE.clone());
    }
 
    #[tokio::test]
    async fn test_get_block_hash() {
        let res = AC_RPC.get_block_hash(Some(1), None).await;
        match  res {
            Some(r) =>  { 
                assert!(true); 
            }
            None => { 
                assert!(false); 
            }
        }
    }
    
    /*#[tokio::test]
    async fn test_get_header() {
        let res = AC_RPC.get_header(None, None).await;
        match  res {
            Some(r) =>  { 
                assert!(true); 
            }
            None => { 
                assert!(false); 
            }
        }
    }*/

    #[tokio::test]
    async fn test_get_runtime_version() {
        let res = AC_RPC.get_runtime_version(None, None).await;
        println!("test_get_runtime_version {}", res.spec_version);
        assert!(res.spec_version == 13)
    }
}

