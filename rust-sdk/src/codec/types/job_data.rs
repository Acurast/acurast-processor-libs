use crate::codec::types::acurast::job_registration::JobRegistration;

#[derive(uniffi::Record, Debug)]
pub struct JobData {
    pub registration: JobRegistration,
    pub key_id: Option<Vec<u8>>,
}