use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("ClickHouse error: {0}")]
    ClickHouse(#[from] clickhouse::error::Error),

    #[error("Event Store error: {0}")]
    EventStore(String),

    #[error("Projection error: {0}")]
    Projection(String),

    #[error("Query error: {0}")]
    Query(String),
}
