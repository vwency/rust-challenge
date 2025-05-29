use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("ClickHouse error: {0}")]
    ClickHouse(#[from] clickhouse::error::Error),
}
