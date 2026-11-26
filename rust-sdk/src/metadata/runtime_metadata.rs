use super::reader::{ReadResult, ScaleReader};
use std::collections::HashMap;

/// The parts of a runtime's metadata (V14 or V15) needed to decode calls: the type registry and
/// the pallets with their call types. Same model as `acurast.codec.metadata.RuntimeMetadata` of
/// the Kotlin SDK.
pub(crate) struct Metadata {
    pub(crate) types: HashMap<u32, TypeInfo>,
    pub(crate) pallets: Vec<Pallet>,
}

pub(crate) struct Pallet {
    pub(crate) name: String,
    pub(crate) index: u8,
    pub(crate) call_type: Option<u32>,
}

pub(crate) struct TypeInfo {
    pub(crate) path: Vec<String>,
    pub(crate) def: TypeDef,
}

pub(crate) struct Field {
    pub(crate) name: Option<String>,
    pub(crate) type_id: u32,
    pub(crate) type_name: Option<String>,
}

pub(crate) struct Variant {
    pub(crate) name: String,
    pub(crate) index: u8,
    pub(crate) fields: Vec<Field>,
}

pub(crate) enum TypeDef {
    Composite(Vec<Field>),
    Variant(Vec<Variant>),
    Sequence(u32),
    Array { length: u32, type_id: u32 },
    Tuple(Vec<u32>),
    Primitive(PrimitiveKind),
    Compact,
    BitSequence,
}

/// Primitive types, in the order of their SCALE index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PrimitiveKind {
    Bool,
    Char,
    Str,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    I8,
    I16,
    I32,
    I64,
    I128,
    I256,
}

impl PrimitiveKind {
    const ALL: [PrimitiveKind; 15] = [
        Self::Bool,
        Self::Char,
        Self::Str,
        Self::U8,
        Self::U16,
        Self::U32,
        Self::U64,
        Self::U128,
        Self::U256,
        Self::I8,
        Self::I16,
        Self::I32,
        Self::I64,
        Self::I128,
        Self::I256,
    ];

    pub(crate) fn byte_count(self) -> usize {
        match self {
            Self::Bool | Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::Char | Self::U32 | Self::I32 => 4,
            Self::U64 | Self::I64 => 8,
            Self::U128 | Self::I128 => 16,
            Self::U256 | Self::I256 => 32,
            Self::Str => 0,
        }
    }

    pub(crate) fn is_signed(self) -> bool {
        matches!(
            self,
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128 | Self::I256
        )
    }
}

const METADATA_MAGIC: u32 = 0x6174_656d; // "meta", little-endian
const SUPPORTED_VERSIONS: [u8; 2] = [14, 15];

impl Metadata {
    /// Decodes the `RuntimeMetadataPrefixed` returned by `state_getMetadata`.
    pub(crate) fn decode(bytes: &[u8]) -> ReadResult<Self> {
        let mut reader = ScaleReader::new(bytes);
        if reader.read_u32()? != METADATA_MAGIC {
            return Err("Not runtime metadata".into());
        }
        let version = reader.read_u8()?;
        if !SUPPORTED_VERSIONS.contains(&version) {
            return Err(format!("Unsupported metadata version {version}"));
        }

        let types = reader.read_list(read_portable_type)?.into_iter().collect();
        let pallets = reader.read_list(|r| read_pallet(r, version >= 15))?;

        Ok(Self { types, pallets })
    }

    pub(crate) fn type_info(&self, id: u32) -> ReadResult<&TypeInfo> {
        self.types
            .get(&id)
            .ok_or_else(|| format!("Unknown type {id}"))
    }

    pub(crate) fn pallet(&self, index: u8) -> Option<&Pallet> {
        self.pallets.iter().find(|p| p.index == index)
    }
}

fn read_portable_type(r: &mut ScaleReader) -> ReadResult<(u32, TypeInfo)> {
    let id = r.read_compact_u32()?;
    let path = r.read_list(|r| r.read_string())?;
    // Type parameters.
    r.read_list(|r| {
        r.read_string()?;
        r.read_optional(|r| r.read_compact_u32())
    })?;
    let def = read_type_def(r)?;
    skip_docs(r)?;

    Ok((id, TypeInfo { path, def }))
}

fn read_type_def(r: &mut ScaleReader) -> ReadResult<TypeDef> {
    Ok(match r.read_u8()? {
        0 => TypeDef::Composite(r.read_list(read_field)?),
        1 => TypeDef::Variant(r.read_list(read_variant)?),
        2 => TypeDef::Sequence(r.read_compact_u32()?),
        3 => TypeDef::Array {
            length: r.read_u32()?,
            type_id: r.read_compact_u32()?,
        },
        4 => TypeDef::Tuple(r.read_list(|r| r.read_compact_u32())?),
        5 => {
            let index = r.read_u8()?;
            let kind = PrimitiveKind::ALL
                .get(usize::from(index))
                .ok_or_else(|| format!("Unknown primitive {index}"))?;
            TypeDef::Primitive(*kind)
        }
        6 => {
            r.read_compact_u32()?; // compact type
            TypeDef::Compact
        }
        7 => {
            r.read_compact_u32()?; // bit store type
            r.read_compact_u32()?; // bit order type
            TypeDef::BitSequence
        }
        kind => return Err(format!("Unknown type definition {kind}")),
    })
}

fn read_field(r: &mut ScaleReader) -> ReadResult<Field> {
    let name = r.read_optional(|r| r.read_string())?;
    let type_id = r.read_compact_u32()?;
    let type_name = r.read_optional(|r| r.read_string())?;
    skip_docs(r)?;

    Ok(Field {
        name,
        type_id,
        type_name,
    })
}

fn read_variant(r: &mut ScaleReader) -> ReadResult<Variant> {
    let name = r.read_string()?;
    let fields = r.read_list(read_field)?;
    let index = r.read_u8()?;
    skip_docs(r)?;

    Ok(Variant {
        name,
        index,
        fields,
    })
}

fn read_pallet(r: &mut ScaleReader, with_docs: bool) -> ReadResult<Pallet> {
    let name = r.read_string()?;
    r.read_optional(skip_storage)?;
    let call_type = r.read_optional(|r| r.read_compact_u32())?;
    r.read_optional(|r| r.read_compact_u32())?; // event type
    r.read_list(skip_constant)?;
    r.read_optional(|r| r.read_compact_u32())?; // error type
    let index = r.read_u8()?;
    if with_docs {
        skip_docs(r)?;
    }

    Ok(Pallet {
        name,
        index,
        call_type,
    })
}

fn skip_storage(r: &mut ScaleReader) -> ReadResult<()> {
    r.read_string()?; // prefix
    r.read_list(skip_storage_entry)?;
    Ok(())
}

fn skip_storage_entry(r: &mut ScaleReader) -> ReadResult<()> {
    r.read_string()?; // name
    r.read_u8()?; // modifier
    match r.read_u8()? {
        0 => {
            r.read_compact_u32()?;
        }
        1 => {
            r.read_list(|r| r.read_u8())?; // hashers
            r.read_compact_u32()?; // key
            r.read_compact_u32()?; // value
        }
        kind => return Err(format!("Unknown storage entry type {kind}")),
    }
    r.read_byte_array()?; // default value
    skip_docs(r)
}

fn skip_constant(r: &mut ScaleReader) -> ReadResult<()> {
    r.read_string()?; // name
    r.read_compact_u32()?; // type
    r.read_byte_array()?; // value
    skip_docs(r)
}

fn skip_docs(r: &mut ScaleReader) -> ReadResult<()> {
    r.read_list(|r| r.read_string())?;
    Ok(())
}
