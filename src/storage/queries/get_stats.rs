use async_trait::async_trait;
use clickhouse::Client;

use crate::model::UserStats;
use crate::storage::errors::StorageError;

#[async_trait]
pub trait GetStatsQuery {
    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError>;
}

pub struct ClickHouseGetStatsQuery {
    client: Client,
}

impl ClickHouseGetStatsQuery {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl GetStatsQuery for ClickHouseGetStatsQuery {
    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError> {
        let mut cursor = self.client
            .query(
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
            )
            .fetch::<UserStats>()
            .map_err(StorageError::ClickHouse)?;

        let mut stats = Vec::new();
        while let Some(stat) = cursor.next().await.map_err(StorageError::ClickHouse)? {
            stats.push(stat);
        }
        Ok(stats)
    }
}
