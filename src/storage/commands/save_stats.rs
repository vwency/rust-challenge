use async_trait::async_trait;
use clickhouse::Client;

use crate::model::UserStats;
use crate::storage::errors::StorageError;

#[async_trait]
pub trait SaveStatsCommand {
    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError>;
}

pub struct ClickHouseSaveStatsCommand {
    client: Client,
}

impl ClickHouseSaveStatsCommand {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SaveStatsCommand for ClickHouseSaveStatsCommand {
    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError> {
        if stats.is_empty() {
            return Ok(());
        }

        self.client
            .query("TRUNCATE TABLE user_stats")
            .execute()
            .await
            .map_err(StorageError::ClickHouse)?;

        let mut insert = self.client.insert("user_stats")?;

        for s in stats {
            insert.write(s).await.map_err(StorageError::ClickHouse)?;
        }

        insert.end().await.map_err(StorageError::ClickHouse)?;
        Ok(())
    }
}
