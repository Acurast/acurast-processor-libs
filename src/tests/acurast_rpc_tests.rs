#![cfg(test)]

use crate::acurast_rpc::{AcurastRpc, AcurastRpcImpl};
use crate::big_uint::BigIntToInt;
use crate::codec::types::acurast::job_identifier::JobIdentifier;
use crate::engine::engine::RpcEngine;
use crate::engine::http::http_client::HttpClient;
use crate::engine::http::http_client_impl::HttpClientImpl;
use crate::engine::http_engine::HttpRpcEngine;
use crate::Result;
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref CLIENT: Arc<dyn HttpClient + Send + Sync> = Arc::new(HttpClientImpl::new());
    static ref ENGINE: Arc<dyn RpcEngine> = Arc::new(HttpRpcEngine::new("https://acurast-devnet-ws.prod.gke.papers.tech".to_string(), CLIENT.clone()));
    static ref AC_RPC: AcurastRpcImpl = AcurastRpcImpl::new(ENGINE.clone());
}

#[tokio::test]
async fn test_get_block_hash() -> Result<()> {
    let res = AC_RPC.get_block_hash(Some(1), None).await;
    res.map(|_| ())
}

#[tokio::test]
async fn test_get_header() -> Result<()> {
    let res = AC_RPC.get_header(None, None).await;
    res.map(|_| ())
}

#[tokio::test]
async fn test_get_runtime_version() {
    let res = AC_RPC.get_runtime_version(None, None).await.unwrap();
    println!("test_get_runtime_version {}", res.spec_version);
    assert!(res.spec_version != 0)
}

#[tokio::test]
async fn test_get_api_version() {
    let res = AC_RPC.get_api_version(None, None).await.unwrap();
    println!("test_get_api_version {}", res);
    assert!(res != 0)
}

#[tokio::test]
async fn test_get_asset() {
    let storage = AC_RPC.storage(1);
    let asset = storage.get_asset(12, None, None).await;
    println!("test_get_asset {:?}", asset);
    assert!(asset.is_some())
}

#[tokio::test]
async fn test_get_job_environment() {
    let storage = AC_RPC.storage(0);
    let account_hex = "5a6ccbc98ee1293ea246e639b8b30a0454e3bf6bdb2318b8aec27b26e606db8d";
    let job_source_hex = "e2a2afc119e466aafac48cf5da2a8995827b1c797c08ba27540288d53e36c914";
    let decoded_bytes = hex::decode(job_source_hex).expect("Decoding failed");
    let decoded_account = hex::decode(account_hex).expect("Decoding failed");
    let environment = storage
        .get_job_environment(
            JobIdentifier::from_integer(5005, decoded_bytes),
            &decoded_account,
            None,
            None,
        )
        .await;
    println!("test_get_job_environment {:?}", environment);
    assert!(environment.is_some())
}

#[tokio::test]
async fn test_get_storage() {
    let storage = AC_RPC.storage(0);
    let hex_str = "5a6ccbc98ee1293ea246e639b8b30a0454e3bf6bdb2318b8aec27b26e606db8d";

    let decoded_bytes = hex::decode(hex_str).expect("Decoding failed");
    println!("test_get_storage: {:?}", decoded_bytes);
    let byte_slice: &[u8] = &decoded_bytes;
    let identifier = storage.get_manager_identifier(byte_slice, None, None).await;
    match identifier {
        Ok(Some(r)) => {
            println!("identifier! {}", r);

            match storage.get_asset(r, None, None).await {
                Some(address) => {
                    println!("address owner: {:?}", address.owner);
                    let account_info = storage
                        .get_account_info(&address.owner.bytes, None, None)
                        .await;
                    println!("account_info: {:?}", account_info);
                }
                None => {
                    panic!();
                }
            }
        }
        _ => {
            panic!();
        }
    }
}

#[tokio::test]
async fn test_get_attestation() {
    let storage = AC_RPC.storage(0);
    let hex_str = "fff7dbe4dc65dd43bb4209aafd5a34d6a847547735894c2d2a5cedd4ab072133";

    let decoded_bytes = hex::decode(hex_str).expect("Decoding failed");
    println!("test_get_attestation: {:?}", decoded_bytes);
    let byte_slice: &[u8] = &decoded_bytes;
    let attestation = storage.get_attestation(byte_slice, None, None).await;
    match attestation {
        Some(r) => {
            println!("attestation owner: {:?}", r.content);
        }
        None => {
            println!("No Attested");
            panic!();
        }
    }
}

#[tokio::test]
async fn test_get_assigned_jobs() {
    let storage = AC_RPC.storage(0);
    //5a6ccbc98ee1293ea246e639b8b30a0454e3bf6bdb2318b8aec27b26e606db8d
    //8edb923e73edfc8ef26bc373653d1cc556ba8a9e3736efef2375a64518427908
    let account_hex = "967dd37bb56eb6352d956b4ad4c557118ba90b0d6ccf10fa30cbaed39940d298";
    let decoded_bytes = hex::decode(account_hex).expect("Decoding failed");

    let job_assignment = storage.get_assigned_jobs(&decoded_bytes, None, None).await;
    println!(
        "job_assigments:job_id {:?}",
        job_assignment.first().unwrap().job_id.id.to_u8a()
    );
    println!(
        "job_assigments:origin {:?}",
        job_assignment.first().unwrap().job_id.origin.to_u8a()
    );
    println!(
        "job_assigments {}",
        job_assignment.first().unwrap().job_id.id
    );
    let job_id = &job_assignment.first().unwrap().job_id;
    let job_registration = storage
        .get_job_registration(job_id.clone(), None, None)
        .await;
    println!("job_registration {:?}", job_registration);
}

#[tokio::test]
async fn test_pool_ids() {
    println!("test_pool_ids start");
    let storage = AC_RPC.storage(0);
    let names: Vec<String> = vec!["v1_cpu_single_core______".to_string()];
    let pool_ids = storage
        .get_metric_pool_ids(names, None, None)
        .await
        .unwrap();
    println!("pool_ids: {:?}", pool_ids);
    assert!(!pool_ids.is_empty());
}

#[tokio::test]
async fn test_pool() {
    println!("test_pool_ids start");
    let storage = AC_RPC.storage(0);
    let pools = storage.get_metric_pools(vec![1], None, None).await.unwrap();
    println!("pool_ids: {:?}", pools);
    assert!(!pools.is_empty());
}

/*#[tokio::test]
async fn test_get_job_registration() {
    let storage = AC_RPC.storage(0);
    let job_registration = storage.get_job_registration(JobIdentifier::empty(), None, None).await;
    println!("job_registration {:?}", job_registration);

}*/
