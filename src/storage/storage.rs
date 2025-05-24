use crate::model::{Transfer, UserStats};
use clickhouse_rs::{types::Block, Pool};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug)]
pub struct ClickHouseStorage {
    pool: Pool,
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("ClickHouse error: {0}")]
    ClickHouse(#[from] clickhouse_rs::errors::Error),

    #[allow(dead_code)]
    #[error("Storage error: {0}")]
    Generic(String),
}

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError>;
    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError>;
    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError>;
}

impl ClickHouseStorage {
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        println!("Подключение к ClickHouse по адресу: {}", database_url);

        let pool = Pool::new(database_url);

        let mut client = match pool.get_handle().await {
            Ok(client) => {
                println!("Подключение установлено успешно");
                client
            }
            Err(e) => {
                eprintln!("Ошибка подключения: {}", e);
                return Err(StorageError::ClickHouse(e));
            }
        };

        match client.query("SHOW DATABASES").fetch_all().await {
            Ok(block) => {
                println!("Доступные базы данных:");
                for row in block.rows() {
                    let db: String = row.get(0)?;
                    println!("- {}", db);
                }
            }
            Err(e) => {
                eprintln!("Ошибка при получении списка баз данных: {}", e);
                return Err(StorageError::ClickHouse(e));
            }
        }

        match client.query("SHOW TABLES FROM default").fetch_all().await {
            Ok(block) => {
                println!("Таблицы в базе default:");
                for row in block.rows() {
                    let table: String = row.get(0)?;
                    println!("- {}", table);
                }
            }
            Err(e) => {
                eprintln!("Ошибка при получении списка таблиц: {}", e);
                return Err(StorageError::ClickHouse(e));
            }
        }

        println!("Создаем таблицу transfers...");
        if let Err(e) = client.execute("
            CREATE TABLE IF NOT EXISTS default.transfers (
                ts UInt64,
                `from` String,
                `to` String,
                amount Float64,
                usd_price Float64
            ) ENGINE = MergeTree()
            ORDER BY (ts, from, to)
        ").await {
            eprintln!("Ошибка создания таблицы transfers: {}", e);
            return Err(StorageError::ClickHouse(e));
        }

        println!("Создаем таблицу user_stats...");
        if let Err(e) = client.execute("
            CREATE TABLE IF NOT EXISTS default.user_stats (
                address String,
                total_volume Float64,
                avg_buy_price Float64,
                avg_sell_price Float64,
                max_balance Float64
            ) ENGINE = ReplacingMergeTree()
            ORDER BY address
        ").await {
            eprintln!("Ошибка создания таблицы user_stats: {}", e);
            return Err(StorageError::ClickHouse(e));
        }

        println!("Таблицы созданы успешно");
        Ok(Self { pool })
    }
}


#[async_trait::async_trait]
impl Storage for ClickHouseStorage {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError> {
        if transfers.is_empty() {
            return Ok(());
        }

        let mut client = self.pool.get_handle().await?;

        println!("Очищаем старые данные из таблицы transfers...");
        client.execute("TRUNCATE TABLE transfers").await?;

        println!("Подготавливаем данные для вставки...");
        let ts_vec: Vec<u64> = transfers.iter().map(|t| t.ts).collect();
        let from_vec: Vec<String> = transfers.iter().map(|t| t.from.clone()).collect();
        let to_vec: Vec<String> = transfers.iter().map(|t| t.to.clone()).collect();
        let amount_vec: Vec<f64> = transfers.iter().map(|t| t.amount).collect();
        let price_vec: Vec<f64> = transfers.iter().map(|t| t.usd_price).collect();

        let mut block = Block::new();
        block = block
            .column("ts", ts_vec)
            .column("from", from_vec)
            .column("to", to_vec)
            .column("amount", amount_vec)
            .column("usd_price", price_vec);

        println!("Вставляем {} записей...", transfers.len());
        client.insert("transfers", block).await?;
        println!("Трансферы успешно сохранены");

        Ok(())
    }

    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError> {
        if stats.is_empty() {
            return Ok(());
        }

        let mut client = self.pool.get_handle().await?;

        println!("Очищаем старые данные из таблицы user_stats...");
        client.execute("TRUNCATE TABLE user_stats").await?;

        println!("Подготавливаем статистику для вставки...");
        let addr_vec: Vec<String> = stats.iter().map(|s| s.address.clone()).collect();
        let volume_vec: Vec<f64> = stats.iter().map(|s| s.total_volume).collect();
        let buy_price_vec: Vec<f64> = stats.iter().map(|s| s.avg_buy_price).collect();
        let sell_price_vec: Vec<f64> = stats.iter().map(|s| s.avg_sell_price).collect();
        let max_balance_vec: Vec<f64> = stats.iter().map(|s| s.max_balance).collect();

        let mut block = Block::new();
        block = block
            .column("address", addr_vec)
            .column("total_volume", volume_vec)
            .column("avg_buy_price", buy_price_vec)
            .column("avg_sell_price", sell_price_vec)
            .column("max_balance", max_balance_vec);

        println!("Вставляем {} записей статистики...", stats.len());
        client.insert("user_stats", block).await?;
        println!("Статистика успешно сохранена");

        Ok(())
    }

    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError> {
        let mut client = self.pool.get_handle().await?;

        println!("Читаем статистику из базы данных...");
        let block = client
            .query("SELECT address, total_volume, avg_buy_price, avg_sell_price, max_balance FROM user_stats ORDER BY total_volume DESC")
            .fetch_all()
            .await?;

        let mut stats = Vec::new();

        for row in block.rows() {
            let address: String = row.get("address")?;
            let total_volume: f64 = row.get("total_volume")?;
            let avg_buy_price: f64 = row.get("avg_buy_price")?;
            let avg_sell_price: f64 = row.get("avg_sell_price")?;
            let max_balance: f64 = row.get("max_balance")?;

            stats.push(UserStats {
                address,
                total_volume,
                avg_buy_price,
                avg_sell_price,
                max_balance,
                max_balance_1h: 0.0,
                max_balance_24h: 0.0,
                max_balance_7d: 0.0,
            });
        }

        println!("Прочитано {} записей из базы данных", stats.len());
        Ok(stats)
    }
}

#[derive(Debug)]
pub struct MockStorage {
    transfers: Mutex<Vec<Transfer>>,
    stats: Mutex<Vec<UserStats>>,
}

impl MockStorage {
    pub fn new() -> Self {
        println!("Инициализирован мок-storage");
        Self {
            transfers: Mutex::new(Vec::new()),
            stats: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl Storage for MockStorage {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError> {
        println!("MockStorage: сохраняем {} трансферов", transfers.len());
        *self.transfers.lock().unwrap() = transfers.to_vec();
        Ok(())
    }

    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError> {
        println!("MockStorage: сохраняем {} записей статистики", stats.len());
        let mut locked_stats = self.stats.lock().unwrap();
        *locked_stats = stats.to_vec();
        locked_stats.sort_by(|a, b| b.total_volume.partial_cmp(&a.total_volume).unwrap());
        Ok(())
    }

    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError> {
        let stats = self.stats.lock().unwrap();
        println!("MockStorage: возвращаем {} записей статистики", stats.len());
        Ok(stats.clone())
    }
}
