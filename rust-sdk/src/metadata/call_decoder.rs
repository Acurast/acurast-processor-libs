use super::reader::{ReadResult, ScaleReader};
use super::runtime_metadata::{Field, Metadata, PrimitiveKind, TypeDef};
use super::{CallArgument, CallValue, DecodedCall};

const MAX_DEPTH: usize = 64;
const RUNTIME_CALL: &str = "RuntimeCall";
const ACCOUNT_ID: &str = "AccountId32";

/// Decodes the SCALE-encoded call of a transaction with `metadata`, consuming every byte.
pub(crate) fn decode_call(call: &[u8], metadata: &Metadata) -> ReadResult<DecodedCall> {
    let mut decoder = CallDecoder {
        metadata,
        reader: ScaleReader::new(call),
    };
    let decoded = decoder.decode_call(0)?;
    match decoder.reader.remaining() {
        0 => Ok(decoded),
        remaining => Err(format!("Trailing call data: {remaining} bytes")),
    }
}

struct CallDecoder<'a> {
    metadata: &'a Metadata,
    reader: ScaleReader<'a>,
}

impl CallDecoder<'_> {
    fn decode_call(&mut self, depth: usize) -> ReadResult<DecodedCall> {
        let index = self.reader.read_u8()?;
        let metadata = self.metadata;
        let pallet = metadata
            .pallet(index)
            .ok_or_else(|| format!("Unknown pallet {index}"))?;
        let call_type = pallet
            .call_type
            .ok_or_else(|| format!("Pallet {} has no calls", pallet.name))?;
        let (name, arguments, _) = self.decode_variant(call_type, depth)?;

        Ok(DecodedCall {
            id: call_id(&pallet.name, &name),
            pallet: pallet.name.clone(),
            name,
            arguments,
        })
    }

    fn decode(&mut self, type_id: u32, depth: usize) -> ReadResult<CallValue> {
        if depth > MAX_DEPTH {
            return Err("Nesting too deep".into());
        }
        let metadata = self.metadata;
        let type_info = metadata.type_info(type_id)?;
        if type_info.path.last().map(String::as_str) == Some(RUNTIME_CALL) {
            return Ok(CallValue::Call {
                call: self.decode_call(depth + 1)?,
            });
        }

        Ok(match &type_info.def {
            TypeDef::Composite(fields) => {
                let fields = self.decode_fields(fields, depth)?;
                let is_account = type_info.path.last().map(String::as_str) == Some(ACCOUNT_ID);
                match fields.as_slice() {
                    [CallArgument {
                        value: CallValue::Bytes { value },
                        ..
                    }] if is_account => CallValue::Account {
                        account_id: value.clone(),
                    },
                    _ => CallValue::Composite { fields },
                }
            }
            TypeDef::Variant(_) => {
                let (name, fields, type_path) = self.decode_variant(type_id, depth)?;
                CallValue::Variant {
                    name,
                    fields,
                    type_path,
                }
            }
            TypeDef::Sequence(element) => {
                let length = self.reader.read_compact_usize()?;
                self.decode_sequence(length, *element, depth)?
            }
            TypeDef::Array { length, type_id } => {
                self.decode_sequence(*length as usize, *type_id, depth)?
            }
            TypeDef::Tuple(types) => CallValue::Composite {
                fields: types
                    .iter()
                    .map(|t| {
                        Ok(CallArgument {
                            name: None,
                            type_name: None,
                            value: self.decode(*t, depth + 1)?,
                        })
                    })
                    .collect::<ReadResult<_>>()?,
            },
            TypeDef::Primitive(kind) => self.decode_primitive(*kind)?,
            TypeDef::Compact => CallValue::Number {
                value: self.reader.read_compact()?.to_string(),
            },
            TypeDef::BitSequence => return Err("Bit sequences are not supported".into()),
        })
    }

    fn decode_variant(
        &mut self,
        type_id: u32,
        depth: usize,
    ) -> ReadResult<(String, Vec<CallArgument>, Vec<String>)> {
        let metadata = self.metadata;
        let type_info = metadata.type_info(type_id)?;
        let TypeDef::Variant(variants) = &type_info.def else {
            return Err(format!("Type {type_id} is not an enum"));
        };
        let index = self.reader.read_u8()?;
        let variant = variants
            .iter()
            .find(|v| v.index == index)
            .ok_or_else(|| format!("Unknown variant {index} of type {type_id}"))?;
        let fields = self.decode_fields(&variant.fields, depth)?;

        Ok((variant.name.clone(), fields, type_info.path.clone()))
    }

    fn decode_fields(&mut self, fields: &[Field], depth: usize) -> ReadResult<Vec<CallArgument>> {
        fields
            .iter()
            .map(|field| {
                Ok(CallArgument {
                    name: field.name.clone(),
                    type_name: field.type_name.clone(),
                    value: self.decode(field.type_id, depth + 1)?,
                })
            })
            .collect()
    }

    fn decode_sequence(
        &mut self,
        length: usize,
        element: u32,
        depth: usize,
    ) -> ReadResult<CallValue> {
        let metadata = self.metadata;
        if let TypeDef::Primitive(PrimitiveKind::U8) = metadata.type_info(element)?.def {
            return Ok(CallValue::Bytes {
                value: self.reader.read_bytes(length)?,
            });
        }
        if length > self.reader.remaining() {
            return Err("Sequence longer than the remaining data".into());
        }
        let items = (0..length)
            .map(|_| self.decode(element, depth + 1))
            .collect::<ReadResult<_>>()?;

        Ok(CallValue::Sequence { items })
    }

    fn decode_primitive(&mut self, kind: PrimitiveKind) -> ReadResult<CallValue> {
        Ok(match kind {
            PrimitiveKind::Bool => CallValue::Bool {
                value: self.reader.read_bool()?,
            },
            PrimitiveKind::Str => CallValue::Text {
                value: self.reader.read_string()?,
            },
            PrimitiveKind::Char => {
                let value = self.reader.read_u32()?;
                let char = char::from_u32(value).ok_or("Invalid char")?;
                CallValue::Text {
                    value: char.to_string(),
                }
            }
            kind if kind.is_signed() => CallValue::Number {
                value: self.reader.read_signed(kind.byte_count())?.to_string(),
            },
            kind => CallValue::Number {
                value: self.reader.read_unsigned(kind.byte_count())?.to_string(),
            },
        })
    }
}

/// `pallet.call` in polkadot-js notation: `Balances` + `transfer_keep_alive` → `balances.transferKeepAlive`.
fn call_id(pallet: &str, name: &str) -> String {
    let mut pallet_chars = pallet.chars();
    let pallet = match pallet_chars.next() {
        Some(first) => first.to_lowercase().chain(pallet_chars).collect(),
        None => String::new(),
    };
    let name: String = name
        .split('_')
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                return part.to_string();
            }
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect();

    format!("{pallet}.{name}")
}
