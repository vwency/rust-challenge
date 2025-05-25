use async_trait::async_trait;
use crate::model::{Transfer, UserStats};
use crate::storage::errors::StorageError;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError>;
    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError>;
    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError>;
}
