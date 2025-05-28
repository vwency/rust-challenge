use async_trait::async_trait;
use clickhouse::Client;

use crate::model::Transfer;
use crate::storage::errors::StorageError;

#[async_trait]
pub trait SaveTransfersCommand {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError>;
}

pub struct ClickHouseSaveTransfersCommand {
    client: Client,
}

impl ClickHouseSaveTransfersCommand {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SaveTransfersCommand for ClickHouseSaveTransfersCommand {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError> {
        if transfers.is_empty() {
            return Ok(());
        }

        self.client
            .query("TRUNCATE TABLE transfers")
            .execute()
            .await
            .map_err(StorageError::ClickHouse)?;

        let mut insert = self.client.insert("transfers")?;

        for t in transfers {
            insert.write(t).await.map_err(StorageError::ClickHouse)?;
        }

        insert.end().await.map_err(StorageError::ClickHouse)?;
        Ok(())
    }
}
