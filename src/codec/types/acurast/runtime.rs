use std::{error::Error, io::{Cursor, Read}};

#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    NodeJS,
    NodeJSWithBundle,
    Shell,
}

impl Runtime {
    pub fn id(&self) -> u8 {
        match self {
            Runtime::NodeJS => 0,
            Runtime::NodeJSWithBundle => 1,
            Runtime::Shell => 2,
        }
    }

    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let mut id_buf = [0u8; 1]; // Buffer for the ID byte
        cursor.read_exact(&mut id_buf)?; // Read 1 byte
        let id = id_buf[0];

        match id {
            0 => Ok(Runtime::NodeJS),
            1 => Ok(Runtime::NodeJSWithBundle),
            2 => Ok(Runtime::Shell),
            _ => Err(format!("Unknown Runtime ID: {}", id).into()),
        }
    }
}