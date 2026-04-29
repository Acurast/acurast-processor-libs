use thiserror::Error;

#[derive(Error, Debug, uniffi::Enum)]
pub enum AcurastError {
    #[error("Acurast runtime error")]
    RuntimeError(String),
    #[error("Error parsing bytes")]
    ParsingError,
    #[error("Unknown error")]
    Unknown,
}

impl From<String> for AcurastError {
    fn from(s: String) -> Self {
        AcurastError::RuntimeError(s)
    }
}
