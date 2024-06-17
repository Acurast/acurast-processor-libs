use std::str::FromStr;
use std::sync::Arc;
use serde_json::{json, Value};
use num_bigint::{BigUint, ToBigUint};
use num_traits::Num;
use crate::engine::engine::{rpc_engine_request, RpcEngine};
use crate::types::header::Header;

pub struct Chain;

impl Chain {
    pub fn new() -> Self {
        Chain {  }
    }
}

impl Chain {
    pub async fn get_block_hash(
        &self,
        block_number: Option<i64>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let params = json!([block_number]);
        let response: Value = rpc_engine_request(engine, "chain_getBlockHash", params, timeout).await?;
        if let Some(result) = response.get("result") {
            if let Some(result_str) = result.as_str() {
                return Ok(Some(result_str.to_string()));
            }
        }
        Ok(None)
    }

    pub async fn get_header(
        &self,
        block_hash: Option<Vec<u8>>,
        timeout: Option<u64>,
        engine: &Arc<dyn RpcEngine>,
    ) -> Result<Header, Box<dyn std::error::Error>> {
        let params = json!([block_hash.map(|b| hex::encode(b))]);
        let response = rpc_engine_request(engine, "chain_getHeader", params, timeout).await?;
        let result = response.get("result").and_then(|r| r.as_object()).ok_or("Invalid response")?;
        let parent_hash = result.get("parentHash").and_then(|h| h.as_str()).ok_or("Missing parentHash")?.to_string();
        let number_str = result.get("number").and_then(|n| n.as_str()).ok_or("Missing number")?;
        let number = BigUint::from_str_radix(number_str.trim_start_matches("0x"), 16).map_err(|e| format!("Invalid number format: {}", e))?;
        let state_root = result.get("stateRoot").and_then(|s| s.as_str()).ok_or("Missing stateRoot")?.to_string();
        let extrinsics_root = result.get("extrinsicsRoot").and_then(|e| e.as_str()).ok_or("Missing extrinsicsRoot")?.to_string();
        Ok(Header::new(parent_hash, number, state_root, extrinsics_root))
    }
}