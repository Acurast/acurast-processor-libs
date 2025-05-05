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
