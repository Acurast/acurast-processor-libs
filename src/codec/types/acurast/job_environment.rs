
use std::error::Error;
use std::io::{self, Cursor, Read};
use std::vec::Vec;
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(uniffi::Record, Debug)]
pub struct Variable {
    pub key: String,
    pub value: Vec<u8>,
}

#[derive(uniffi::Record, Debug)]
pub struct JobEnvironment {
    pub public_key: Vec<u8>,
    pub vars: Vec<Variable>,
}

impl Variable {
    pub fn read(
        mut cursor: &mut Cursor<Vec<u8>>,
        _api_version: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let key_len = cursor.read_u32::<LittleEndian>()? as usize;
        let mut key_bytes = vec![0u8; key_len];
        cursor.read_exact(&mut key_bytes)?;
        let key = String::from_utf8(key_bytes)?;

        let value_len = cursor.read_u32::<LittleEndian>()? as usize;
        let mut value_bytes = vec![0u8; value_len];
        cursor.read_exact(&mut value_bytes)?;

        Ok(Variable { key, value: value_bytes })
    }
}

impl JobEnvironment {
    pub fn read(
        mut cursor: &mut Cursor<Vec<u8>>,
        api_version: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let public_key_len = cursor.read_u32::<LittleEndian>()? as usize;
        let mut public_key_bytes = vec![0u8; public_key_len];
        cursor.read_exact(&mut public_key_bytes)?;

        let vars_len = cursor.read_u32::<LittleEndian>()? as usize;
        let mut vars = Vec::with_capacity(vars_len);
        for _ in 0..vars_len {
            vars.push(Variable::read(&mut cursor, api_version)?);
        }

        Ok(JobEnvironment {
            public_key: public_key_bytes,
            vars,
        })
    }
}
