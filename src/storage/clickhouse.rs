use crate::model::{Transfer, UserStats};
use clickhouse_rs::{types::Block, Pool};
use crate::storage::{Storage, errors::StorageError};  // Импортируем ошибку из errors.rs
use async_trait::async_trait;
use anyhow::Context;

#[derive(Debug)]
pub struct ClickHouseStorage {
    pub pool: Pool,
}

impl ClickHouseStorage {
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        println!("Подключение к ClickHouse по адресу: {}", database_url);
        let pool = Pool::new(database_url);
        let mut client = pool.get_handle().await.map_err(StorageError::ClickHouse)?;
        client.query("SHOW DATABASES").fetch_all().await?;
        client.query("SHOW TABLES FROM default").fetch_all().await?;
        client.execute("
            CREATE TABLE IF NOT EXISTS default.transfers (
                ts UInt64,
                `from` String,
                `to` String,
                amount Float64,
                usd_price Float64
            ) ENGINE = MergeTree() ORDER BY (ts, from, to)
        ").await?;
        client.execute("
            CREATE TABLE IF NOT EXISTS default.user_stats (
                address String,
                total_volume Float64,
                avg_buy_price Float64,
                avg_sell_price Float64,
                max_balance Float64,
                max_balance_1h Float64,
                max_balance_24h Float64,
                max_balance_7d Float64
            ) ENGINE = ReplacingMergeTree() ORDER BY address
        ").await?;
        println!("Таблицы созданы успешно");
        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for ClickHouseStorage {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError> {
        if transfers.is_empty() {
            return Ok(());
        }
        let mut client = self.pool.get_handle().await?;
        client.execute("TRUNCATE TABLE transfers").await?;
        let block = Block::new()
            .column("ts", transfers.iter().map(|t| t.ts).collect::<Vec<_>>())
            .column("from", transfers.iter().map(|t| t.from.clone()).collect::<Vec<_>>())
            .column("to", transfers.iter().map(|t| t.to.clone()).collect::<Vec<_>>())
            .column("amount", transfers.iter().map(|t| t.amount).collect::<Vec<_>>())
            .column("usd_price", transfers.iter().map(|t| t.usd_price).collect::<Vec<_>>());
        client.insert("transfers", block).await?;
        Ok(())
    }

    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError> {
        if stats.is_empty() {
            return Ok(());
        }
        let mut client = self.pool.get_handle().await?;
        client.execute("TRUNCATE TABLE user_stats").await?;
        let block = Block::new()
            .column("address", stats.iter().map(|s| s.address.clone()).collect::<Vec<_>>())
            .column("total_volume", stats.iter().map(|s| s.total_volume).collect::<Vec<_>>())
            .column("avg_buy_price", stats.iter().map(|s| s.avg_buy_price).collect::<Vec<_>>())
            .column("avg_sell_price", stats.iter().map(|s| s.avg_sell_price).collect::<Vec<_>>())
            .column("max_balance", stats.iter().map(|s| s.max_balance).collect::<Vec<_>>())
            .column("max_balance_1h", stats.iter().map(|s| s.max_balance_1h).collect::<Vec<_>>())
            .column("max_balance_24h", stats.iter().map(|s| s.max_balance_24h).collect::<Vec<_>>())
            .column("max_balance_7d", stats.iter().map(|s| s.max_balance_7d).collect::<Vec<_>>());
        client.insert("user_stats", block).await?;
        Ok(())
    }

    async fn get_stats(&self) -> anyhow::Result<Vec<UserStats>> {
        let mut client = self.pool.get_handle().await
            .context("Failed to get database handle")?;
        let block = client
            .query("SELECT address, total_volume, avg_buy_price, avg_sell_price, max_balance, max_balance_1h, max_balance_24h, max_balance_7d FROM user_stats ORDER BY total_volume DESC")
            .fetch_all()
            .await
            .context("Failed to execute query")?;

        let rows = block.rows();
        let mut stats = Vec::new();

        for row in rows {
            stats.push(UserStats {
                address: row.get("address").context("Missing 'address' column")?,
                total_volume: row.get("total_volume").context("Missing 'total_volume' column")?,
                avg_buy_price: row.get("avg_buy_price").context("Missing 'avg_buy_price' column")?,
                avg_sell_price: row.get("avg_sell_price").context("Missing 'avg_sell_price' column")?,
                max_balance: row.get("max_balance").context("Missing 'max_balance' column")?,
                max_balance_1h: row.get("max_balance_1h").context("Missing 'max_balance_1h' column")?,
                max_balance_24h: row.get("max_balance_24h").context("Missing 'max_balance_24h' column")?,
                max_balance_7d: row.get("max_balance_7d").context("Missing 'max_balance_7d' column")?,
            });
        }

        Ok(stats)
    }
}
