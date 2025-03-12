use thiserror::Error;

#[derive(Debug, Error)]
pub enum BerryError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("keyring: {0}")]
    Keyring(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("pipeline: {0}")]
    Pipeline(String),
    #[error("protocol unsupported: {0}")]
    Unsupported(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BerryError>;
