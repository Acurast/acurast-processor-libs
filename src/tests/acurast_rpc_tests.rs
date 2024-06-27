

#[cfg(test)]
pub mod acurastTests{
    use hex::ToHex;

    use crate::engine::engine::RpcEngine;
    use crate::engine::http::http_client::HttpClient;
    use crate::engine::http_engine::HttpRpcEngine;
    use crate::engine::http::http_client_impl::HttpClientImpl;
    use crate::acurast_rpc::{AcurastRpc, AcurastRpcImpl};
    use std::sync::Arc;

    lazy_static::lazy_static! {
        static ref CLIENT: Arc<dyn HttpClient + Send + Sync> = Arc::new(HttpClientImpl::new());
        static ref ENGINE: Arc<dyn RpcEngine> = Arc::new(HttpRpcEngine::new("https://processor.canary.acurast.com".to_string(), CLIENT.clone()));
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
    
    #[tokio::test]
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
    }

    #[tokio::test]
    async fn test_get_runtime_version() {
        let res = AC_RPC.get_runtime_version(None, None).await;
        println!("test_get_runtime_version {}", res.spec_version);
        assert!(res.spec_version != 0)
    }

    #[tokio::test]
    async fn test_get_storage() {
        let storage = AC_RPC.storage(0);
        let hex_str = "5a6ccbc98ee1293ea246e639b8b30a0454e3bf6bdb2318b8aec27b26e606db8d";
        
        let decoded_bytes = hex::decode(hex_str).expect("Decoding failed");
        println!("test_get_storage: {:?}",decoded_bytes);
        let byte_slice: &[u8] = &decoded_bytes;
        let identifier = storage.get_manager_identifier(byte_slice, None, None).await;
        match  identifier {
            Some(r) =>  { 
                println!("identifier! {}", r);
                assert!(true); 

                match storage.get_asset(r, None, None).await {
                    Some(address) => {
                        println!("address owner: {:?}", address.owner);
                        assert!(true); 

                        match storage.get_account_info(&address.owner.bytes, None, None).await {
                            Some(account_info) => {
                                println!("account_info: {:?}", account_info);
                                assert!(true); 
                            }
                            None => {
                                assert!(false); 
                            }
                        }
                    }
                    None => {
                        assert!(false); 
                    }
                }
            }
            None => { 
                assert!(false); 
            }
        }
    }
}

