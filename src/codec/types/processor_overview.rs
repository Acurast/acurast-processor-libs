#[derive(uniffi::Record, Debug, Default)]
pub struct ProcessorOverview {
    pub manager_identifier: Option<i32>,
    pub is_attested: bool,
    pub last_heartbeat_timestamp: Option<u64>,
}
