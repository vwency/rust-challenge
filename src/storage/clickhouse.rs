use async_trait::async_trait;
use clickhouse::Client;

use crate::model::{Transfer, UserStats};
use crate::storage::commands::{ClickHouseSaveStatsCommand, ClickHouseSaveTransfersCommand};
use crate::storage::errors::StorageError;

// Импорт трейтов команд и запросов, чтобы методы были видны
use crate::storage::commands::save_transfers::SaveTransfersCommand;
use crate::storage::commands::save_stats::SaveStatsCommand;
use crate::storage::queries::ClickHouseGetStatsQuery;
use crate::storage::queries::get_stats::GetStatsQuery;

// Импорт самого трейта Storage — который реализует ClickHouseStorage
use crate::storage::storage_trait::Storage;

pub struct ClickHouseStorage {
    client: Client,

    save_transfers_cmd: ClickHouseSaveTransfersCommand,
    save_stats_cmd: ClickHouseSaveStatsCommand,
    get_stats_query: ClickHouseGetStatsQuery,
}

impl ClickHouseStorage {
    pub async fn new(client: Client) -> Result<Self, StorageError> {
        // Инициализация таблиц
        client.query("SELECT 1")
            .execute()
            .await
            .map_err(StorageError::ClickHouse)?;

        client.query(r#"
            CREATE TABLE IF NOT EXISTS transfers (
                ts UInt64,
                `from` String,
                `to` String,
                amount Float64,
                usd_price Float64
            ) ENGINE = MergeTree() ORDER BY (ts, `from`, `to`)
        "#).execute()
            .await
            .map_err(StorageError::ClickHouse)?;

        client.query(r#"
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
        "#).execute()
            .await
            .map_err(StorageError::ClickHouse)?;

        Ok(Self {
            save_transfers_cmd: ClickHouseSaveTransfersCommand::new(client.clone()),
            save_stats_cmd: ClickHouseSaveStatsCommand::new(client.clone()),
            get_stats_query: ClickHouseGetStatsQuery::new(client.clone()),
            client,
        })
    }
}

#[async_trait]
impl Storage for ClickHouseStorage {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError> {
        self.save_transfers_cmd.save_transfers(transfers).await
    }

    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError> {
        self.save_stats_cmd.save_stats(stats).await
    }

    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError> {
        self.get_stats_query.get_stats().await
    }
}
