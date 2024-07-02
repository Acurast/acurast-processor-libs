use std::{error::Error, fmt, io::{Cursor, Read}, u128};
use byteorder::{BigEndian, ReadBytesExt};
use hex::FromHex;
use num_bigint::BigUint;
use crate::{big_uint_utils::read_u128, codec::{extrinsic::marketplace::PublicKey, types::{account_id::AccountId32, acurast::job_identifier::JobIdentifier}}, hex::hex_to_ba};

#[derive(uniffi::Record)]
#[derive(Debug)]
struct ExecutionSpecifier;

impl ExecutionSpecifier {
    fn new(kind: ExecutionSpecifierKind) -> Self {
        ExecutionSpecifier {}
    }

    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        // Your implementation here
        Ok(ExecutionSpecifier {})
    }
}

#[derive(uniffi::Record)]
#[derive(Debug)]
struct SLA {
    total: i64,
    met: i64,
}

impl SLA {
    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let total = cursor.read_i64::<BigEndian>()?;
        let met = cursor.read_i64::<BigEndian>()?;
        Ok(SLA { total, met })
    }
}

impl fmt::Display for SLA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SLA(total: {}, met: {})", self.total, self.met)
    }
}

#[derive(Debug)]
enum ExecutionSpecifierKind {
    All,
}

#[derive(uniffi::Record)]
pub struct JobAssignment {
    processor: AccountId32,
    job_id: JobIdentifier,
    slot: u8,
    start_delay: i64,
    fee_per_execution: BigUint,
    acknowledged: bool,
    sla: SLA,
    public_keys: Vec<PublicKey>,
    execution_specifier: ExecutionSpecifier,
}


impl JobAssignment {
    pub fn read(l: Vec<&str>, api_version: u32) -> Result<Self, Box<dyn Error>> {
        // Assuming l[0] and l[1] are hex strings
        println!("JobAssignment::read {:?}", l);
        
        let key_bytes = hex_to_ba(l[0]);
        let value_bytes = hex_to_ba(l[1]);

        let mut key_cursor = Cursor::new(key_bytes);
        key_cursor.set_position(48); // Skip <pallet_name>, <method_name>, <processor_hash>
        let processor = AccountId32::read(&mut key_cursor)?;

        key_cursor.set_position(key_cursor.position() + 16); // Skip <requester_hash>
        let job_id = JobIdentifier::read(&mut key_cursor)?;

        let mut value_cursor = Cursor::new(value_bytes);
        let slot = value_cursor.read_u8()?;
        let start_delay = value_cursor.read_i64::<BigEndian>()?;
        let fee_per_execution = read_u128(&mut value_cursor)?;
        let acknowledged = value_cursor.read_u8()? != 0;
        let sla = SLA::read(&mut value_cursor)?;
        let public_keys = read_list(&mut value_cursor, PublicKey::read)?;

        let execution_specifier = if api_version > 0 {
            ExecutionSpecifier::read(&mut value_cursor)?
        } else {
            ExecutionSpecifier::new(ExecutionSpecifierKind::All)
        };

        Ok(JobAssignment {
            processor,
            job_id,
            slot,
            start_delay,
            fee_per_execution,
            acknowledged,
            sla,
            public_keys,
            execution_specifier,
        })
    }
}


// Helper function to read a list of items from the cursor
fn read_list<T, F>(cursor: &mut Cursor<Vec<u8>>, read_item: F) -> Result<Vec<T>, Box<dyn Error>>
where
    F: Fn(&mut Cursor<Vec<u8>>) -> Result<T, Box<dyn Error>>,
{
    let mut items = Vec::new();
    while (cursor.position() as usize) < cursor.get_ref().len() {
        items.push(read_item(cursor)?);
    }
    Ok(items)
}