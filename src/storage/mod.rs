pub mod clickhouse;
pub mod mock;
pub mod errors;

pub use clickhouse::ClickHouseStorage;
pub use errors::StorageError;
pub use mock::MockStorage;

use async_trait::async_trait;
use crate::model::{Transfer, UserStats};

#[async_trait]
pub trait Storage: Send + Sync {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError>;
    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError>;
    async fn get_stats(&self) -> anyhow::Result<Vec<UserStats>>;
}
