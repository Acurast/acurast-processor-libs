use crate::codec::types::extrinsic::{ExtrinsicCallRust, BytesEncodableRust};
use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct SubmitAttestationExtrinsic;

impl ExtrinsicCallRust for SubmitAttestationExtrinsic {
    fn call_index(&self) -> (u8, u8) {
        (0x28, 0x05)
    }

    fn arguments(&self) -> Arc<dyn BytesEncodableRust> {
        todo!()
    }
}