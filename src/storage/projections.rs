use async_trait::async_trait;
use serde::Deserialize;
use clickhouse::{Client, Row};
use crate::storage::errors::StorageError;

#[derive(Clone, Debug)]
pub struct UserStatsProjection {
    pub address: String,
    pub total_volume: f64,
    pub avg_buy_price: f64,
    pub avg_sell_price: f64,
    pub max_balance: f64,
    pub max_balance_1h: f64,
    pub max_balance_24h: f64,
    pub max_balance_7d: f64,
}

#[derive(Debug, Clone, Deserialize, Row)]
struct UserStatsRow {
    address: String,
    total_volume: f64,
    avg_buy_price: f64,
    avg_sell_price: f64,
    max_balance: f64,
    max_balance_1h: f64,
    max_balance_24h: f64,
    max_balance_7d: f64,
}

#[async_trait]
pub trait UserStatsProjectionTrait {
    async fn load_stats(&self) -> Result<Vec<UserStatsProjection>, StorageError>;
}

pub struct ClickHouseUserStatsProjection {
    client: Client,
}

impl ClickHouseUserStatsProjection {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UserStatsProjectionTrait for ClickHouseUserStatsProjection {
    async fn load_stats(&self) -> Result<Vec<UserStatsProjection>, StorageError> {
        let mut cursor = self
            .client
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
                "#,
            )
            .fetch::<UserStatsRow>()
            .map_err(StorageError::ClickHouse)?;

        let mut stats = Vec::new();
        loop {
            match cursor.next().await {
                Ok(Some(row)) => {
                    stats.push(UserStatsProjection::from(row));
                }
                Ok(None) => break,
                Err(e) => {
                    return Err(StorageError::ClickHouse(e));
                }
            }
        }

        Ok(stats)
    }
}

impl From<UserStatsRow> for UserStatsProjection {
    fn from(row: UserStatsRow) -> Self {
        Self {
            address: row.address,
            total_volume: row.total_volume,
            avg_buy_price: row.avg_buy_price,
            avg_sell_price: row.avg_sell_price,
            max_balance: row.max_balance,
            max_balance_1h: row.max_balance_1h,
            max_balance_24h: row.max_balance_24h,
            max_balance_7d: row.max_balance_7d,
        }
    }
}