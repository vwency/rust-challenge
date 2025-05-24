use thiserror::Error;
use clickhouse_rs::errors::Error as ClickHouseError;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("ClickHouse error: {0}")]
    ClickHouse(#[from] ClickHouseError),

    #[allow(dead_code)]
    #[error("Storage error: {0}")]
    Generic(String),
}

impl From<anyhow::Error> for StorageError {
    fn from(error: anyhow::Error) -> Self {
        StorageError::Generic(error.to_string())
    }
}
