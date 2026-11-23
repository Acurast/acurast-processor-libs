#[derive(uniffi::Record, Debug)]
pub struct ManagementData {
    pub management_endpoint: Option<Vec<u8>>,
    pub migration_proof: Option<Vec<u8>>,
}
