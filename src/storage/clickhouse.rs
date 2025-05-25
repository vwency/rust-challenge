use clickhouse::Client;
use async_trait::async_trait;

use crate::model::{Transfer, UserStats};
use crate::storage::{Storage, errors::StorageError};

pub struct ClickHouseStorage {
    client: Client,
}

impl ClickHouseStorage {
    pub async fn new(client: Client) -> Result<Self, StorageError> {
        client
            .query("SELECT 1")
            .execute()
            .await
            .map_err(StorageError::ClickHouse)?;
        
        client
            .query(r#"
                CREATE TABLE IF NOT EXISTS transfers (
                    ts UInt64,
                    `from` String,
                    `to` String,
                    amount Float64,
                    usd_price Float64
                ) ENGINE = MergeTree() ORDER BY (ts, `from`, `to`)
            "#)
            .execute()
            .await
            .map_err(StorageError::ClickHouse)?;
        
        client
            .query(r#"
                CREATE TABLE IF NOT EXISTS user_stats (
                    address String,
                    total_volume Float64,
                    avg_buy_price Float64,
                    avg_sell_price Float64,
                    max_balance Float64,
                    max_balance_1h Float64,
                    max_balance_24h Float64,
                    max_balance_7d Float64
                ) ENGINE = ReplacingMergeTree() ORDER BY address
            "#)
            .execute()
            .await
            .map_err(StorageError::ClickHouse)?;

        Ok(Self { client })
    }
}

#[async_trait]
impl Storage for ClickHouseStorage {
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

    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError> {
        let mut cursor = self.client.query(
            r#"
            SELECT
                address,
                total_volume,
                avg_buy_price,
                avg_sell_price,
                max_balance,
                max_balance_1h,
                max_balance_24h,
                max_balance_7d
            FROM user_stats
            ORDER BY total_volume DESC
            "#
        ).fetch::<UserStats>()?;

        let mut stats = Vec::new();
        while let Some(stat) = cursor.next().await.map_err(StorageError::ClickHouse)? {
            stats.push(stat);
        }

        Ok(stats)
    }
}
