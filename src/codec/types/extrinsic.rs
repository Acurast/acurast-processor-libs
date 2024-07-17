use std::sync::Arc;

pub trait BytesEncodableRust: Send + Sync {
    fn encode(&self) -> Vec<u8>;
}

pub trait ExtrinsicCallRust: BytesEncodableRust + Send + Sync {
    fn call_index(&self) -> (u8, u8);
    fn arguments(&self) -> Arc<dyn BytesEncodableRust>;
}

impl<T: ExtrinsicCallRust> BytesEncodableRust for T {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![self.call_index().0, self.call_index().1];
        result.extend(self.arguments().encode());
        result
    }
}
