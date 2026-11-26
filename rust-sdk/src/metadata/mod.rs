//! Runtime metadata and call decoding, to show what a transaction does before it is signed.

mod call_decoder;
mod reader;
mod runtime_metadata;

#[cfg(test)]
mod tests;

use crate::{AcurastError, Result};
use runtime_metadata::Metadata;
use std::sync::Arc;

/// The metadata of a runtime version (`state_getMetadata`, V14 or V15), used to decode calls.
#[derive(uniffi::Object)]
pub struct RuntimeMetadata {
    metadata: Metadata,
}

#[uniffi::export]
impl RuntimeMetadata {
    /// Decodes the SCALE-encoded `RuntimeMetadataPrefixed` returned by `state_getMetadata`.
    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Arc<Self>> {
        let metadata = Metadata::decode(&bytes).map_err(AcurastError::RuntimeError)?;
        Ok(Arc::new(Self { metadata }))
    }

    /// Decodes the SCALE-encoded call of a transaction. Fails unless the call decodes exactly,
    /// consuming every byte.
    pub fn decode_call(&self, call: Vec<u8>) -> Result<DecodedCall> {
        call_decoder::decode_call(&call, &self.metadata).map_err(AcurastError::RuntimeError)
    }
}

/// A call decoded with the runtime metadata: `pallet.call(arguments)`.
#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct DecodedCall {
    /// The pallet as named in the metadata, e.g. `Balances`.
    pub pallet: String,
    /// The call as named in the metadata, e.g. `transfer_keep_alive`.
    pub name: String,
    /// The call in polkadot-js notation, e.g. `balances.transferKeepAlive`.
    pub id: String,
    pub arguments: Vec<CallArgument>,
}

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct CallArgument {
    pub name: Option<String>,
    /// The type name declared in the metadata, e.g. `BalanceOf<T>`.
    pub type_name: Option<String>,
    pub value: CallValue,
}

#[derive(uniffi::Enum, Clone, Debug, PartialEq)]
pub enum CallValue {
    Bool {
        value: bool,
    },
    /// An integer of any size, in decimal (negative for signed types).
    Number {
        value: String,
    },
    Text {
        value: String,
    },
    Bytes {
        value: Vec<u8>,
    },
    Account {
        account_id: Vec<u8>,
    },
    Composite {
        fields: Vec<CallArgument>,
    },
    /// An enum value: its variant `name` and `fields`, and the `type_path` of the enum it belongs
    /// to (e.g. `sp_runtime::multiaddress::MultiAddress`), which tells variants with the same name
    /// apart.
    Variant {
        name: String,
        fields: Vec<CallArgument>,
        type_path: Vec<String>,
    },
    Sequence {
        items: Vec<CallValue>,
    },
    Call {
        call: DecodedCall,
    },
}
