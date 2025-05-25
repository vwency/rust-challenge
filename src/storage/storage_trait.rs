use async_trait::async_trait;
use crate::model::{Transfer, UserStats};
use anyhow::Result;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<()>;
    async fn save_stats(&self, stats: &[UserStats]) -> Result<()>;
    async fn get_stats(&self) -> Result<Vec<UserStats>>;
}
