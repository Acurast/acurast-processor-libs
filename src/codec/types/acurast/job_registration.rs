use crate::codec::types::account_id::AccountId32;
use crate::cursor::{read_byte_array, read_list, CursorExt};
use byteorder::{LittleEndian, ReadBytesExt};
use std::error::Error;
use std::io::{self, Cursor};
use std::vec::Vec;

#[derive(uniffi::Record, Debug)]
pub struct JobRegistration {
    // The script to execute. It is a vector of bytes representing a utf8 string. The string needs to be an IPFS URL that points to the script.
    pub script: Vec<u8>,
    // An optional array of AccountId32 allowed to fulfill the job. If the value is None, then all sources are allowed.
    pub allowed_sources: Option<Vec<AccountId32>>,
    // A boolean indicating if only verified sources can fulfill the job. A verified source is one that has provided a valid key attestation.
    pub allow_only_verified_sources: bool,
    // The schedule describing the desired (multiple) execution(s) of the script.
    pub schedule: JobSchedule,
    // Maximum memory bytes used during a single execution of the job.
    pub memory: i32,
    // The modules required for the job.
    pub network_requests: i32,
    // Maximum storage bytes used during the whole period of the job's executions.
    pub storage: i32,
    // Required modules.
    pub required_modules: Vec<JobModule>,
    // Extra parameters.
    pub extra: JobRegistrationExtra,
}

impl JobRegistration {
    pub fn read(
        mut cursor: &mut Cursor<Vec<u8>>,
        api_version: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let script = read_byte_array(&mut cursor)?;
        let allowed_sources = cursor.read_optional(|c| read_list(c, AccountId32::read))?;
        let allow_only_verified_sources = cursor.read_boolean()?;
        let schedule = JobSchedule::read(&mut cursor)?;
        let memory = cursor.read_i32::<LittleEndian>()?;
        let network_requests = cursor.read_i32::<LittleEndian>()?;
        let storage = cursor.read_i32::<LittleEndian>()?;
        let required_modules = read_list(&mut cursor, JobModule::read)?;
        let extra = if api_version > 0 {
            JobRegistrationExtra::read(&mut cursor)?
        } else {
            JobRegistrationExtra::default()
        };

        Ok(JobRegistration {
            script,
            allowed_sources,
            allow_only_verified_sources,
            schedule,
            memory,
            network_requests,
            storage,
            required_modules,
            extra,
        })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct JobSchedule {
    pub duration: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub interval: u64,
    pub max_start_delay: u64,
}

impl JobSchedule {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        Ok(JobSchedule {
            duration: cursor.read_u64::<LittleEndian>()?,
            start_time: cursor.read_u64::<LittleEndian>()?,
            end_time: cursor.read_u64::<LittleEndian>()?,
            interval: cursor.read_u64::<LittleEndian>()?,
            max_start_delay: cursor.read_u64::<LittleEndian>()?,
        })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct JobRegistrationExtra {
    pub requirements: JobRequirements,
}

impl JobRegistrationExtra {
    pub fn read(mut cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        Ok(JobRegistrationExtra {
            requirements: JobRequirements::read(&mut cursor)?,
        })
    }
}

impl Default for JobRegistrationExtra {
    fn default() -> Self {
        JobRegistrationExtra {
            requirements: JobRequirements::default(),
        }
    }
}

#[derive(uniffi::Record, Debug)]
pub struct JobRequirements {
    pub assignment_strategy: AssignmentStrategy,
}

impl JobRequirements {
    pub fn read(mut cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        Ok(JobRequirements {
            assignment_strategy: AssignmentStrategy::read(&mut cursor)?,
        })
    }
}

impl Default for JobRequirements {
    fn default() -> Self {
        JobRequirements {
            assignment_strategy: AssignmentStrategy::Single,
        }
    }
}

#[derive(uniffi::Enum, Debug)]
pub enum AssignmentStrategy {
    Single,
    Competing,
}

impl AssignmentStrategy {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let id = cursor.read_u8()?;
        match id {
            0 => Ok(AssignmentStrategy::Single),
            1 => Ok(AssignmentStrategy::Competing),
            _ => Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown AssignmentStrategy",
            ))),
        }
    }
}

#[derive(uniffi::Enum, Debug)]
pub enum JobModule {
    DataEncryption,
}

impl JobModule {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let id = cursor.read_u8()?;
        match id {
            0 => Ok(JobModule::DataEncryption),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown JobModule {}", id),
            ))),
        }
    }
}
