//! Real data of Acurast Canary, spec 60, encoded by polkadot-js: the same vectors as the tests of
//! the Kotlin SDK and of the processor apps.

use super::{CallArgument, CallValue, RuntimeMetadata};
use flate2::read::GzDecoder;
use std::io::Read;
use std::sync::{Arc, OnceLock};

/// `balances.transferKeepAlive(5G9mkk...VTq, 1_234_567_890_000)`.
const TRANSFER: &str =
    "0a0300b4ce485884bb50a79e159f1c6bb3345869210345f954eb9ea2f5066c3f416a430b5004fb711f01";
/// `utility.batchAll([TRANSFER, system.remark("hello")])`.
const BATCH: &str = "0802080a0300b4ce485884bb50a79e159f1c6bb3345869210345f954eb9ea2f5066c3f416a430b5004fb711f0100001468656c6c6f";
const TRANSFER_DEST: &str = "b4ce485884bb50a79e159f1c6bb3345869210345f954eb9ea2f5066c3f416a43";

fn metadata() -> Arc<RuntimeMetadata> {
    static METADATA: OnceLock<Arc<RuntimeMetadata>> = OnceLock::new();
    METADATA
        .get_or_init(|| {
            let mut bytes = Vec::new();
            GzDecoder::new(&include_bytes!("testdata/canary-metadata.scale.gz")[..])
                .read_to_end(&mut bytes)
                .unwrap();
            RuntimeMetadata::from_bytes(bytes).unwrap()
        })
        .clone()
}

fn bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex).unwrap()
}

fn argument<'a>(arguments: &'a [CallArgument], name: &str) -> &'a CallValue {
    &arguments
        .iter()
        .find(|a| a.name.as_deref() == Some(name))
        .unwrap()
        .value
}

#[test]
fn decode_call_decodes_a_transfer() {
    let call = metadata().decode_call(bytes(TRANSFER)).unwrap();

    assert_eq!(call.pallet, "Balances");
    assert_eq!(call.name, "transfer_keep_alive");
    assert_eq!(call.id, "balances.transferKeepAlive");
    assert_eq!(
        call.arguments
            .iter()
            .map(|a| a.name.as_deref())
            .collect::<Vec<_>>(),
        [Some("dest"), Some("value")]
    );
    // MultiAddress::Id(AccountId32)
    let CallValue::Variant {
        name,
        fields,
        type_path,
    } = argument(&call.arguments, "dest")
    else {
        panic!("{:?}", call.arguments[0]);
    };
    assert_eq!(name, "Id");
    assert_eq!(type_path.last().map(String::as_str), Some("MultiAddress"));
    assert_eq!(
        fields[0].value,
        CallValue::Account {
            account_id: bytes(TRANSFER_DEST)
        }
    );
    assert_eq!(
        argument(&call.arguments, "value"),
        &CallValue::Number {
            value: "1234567890000".into()
        }
    );
    assert!(call.arguments[1]
        .type_name
        .as_deref()
        .unwrap()
        .contains("Balance"));
}

#[test]
fn decode_call_decodes_the_calls_of_a_batch() {
    let call = metadata().decode_call(bytes(BATCH)).unwrap();

    assert_eq!(call.id, "utility.batchAll");
    let CallValue::Sequence { items } = argument(&call.arguments, "calls") else {
        panic!("{:?}", call.arguments);
    };
    let ids: Vec<_> = items
        .iter()
        .map(|item| match item {
            CallValue::Call { call } => call.id.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(ids, ["balances.transferKeepAlive", "system.remark"]);
    let CallValue::Call { call: remark } = &items[1] else {
        unreachable!()
    };
    assert_eq!(
        remark.arguments[0].value,
        CallValue::Bytes {
            value: b"hello".to_vec()
        }
    );
}

#[test]
fn decode_call_rejects_trailing_truncated_and_unknown_data() {
    let metadata = metadata();

    assert!(metadata
        .decode_call(bytes(&format!("{TRANSFER}00")))
        .is_err());
    assert!(metadata
        .decode_call(bytes(&TRANSFER[..TRANSFER.len() - 2]))
        .is_err());
    assert!(metadata.decode_call(vec![0xff, 0x00]).is_err());
    assert!(metadata.decode_call(vec![]).is_err());
}

#[test]
fn from_bytes_rejects_data_that_is_not_metadata() {
    assert!(RuntimeMetadata::from_bytes(vec![]).is_err());
    assert!(RuntimeMetadata::from_bytes(b"meta".to_vec()).is_err());
    // Magic followed by an unsupported version.
    assert!(RuntimeMetadata::from_bytes(vec![0x6d, 0x65, 0x74, 0x61, 13]).is_err());
}

/// Calls a live node: run with `cargo test -p acurast-rust-sdk -- --ignored live_canary`.
#[tokio::test]
#[ignore]
async fn live_canary_metadata_decodes_a_transfer() {
    let client = crate::AcurastRpcClient::new(crate::acurast_rpc(
        "https://processor.canary.acurast.com".into(),
    ));
    let metadata = client.get_metadata(None, None).await.unwrap();

    let call = metadata.decode_call(bytes(TRANSFER)).unwrap();
    assert_eq!(call.id, "balances.transferKeepAlive");
}
