use crate::model::{Transfer, UserStats};
use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Mutex;
use anyhow::{Context, Result};

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

#[async_trait]
impl Storage for MockStorage {
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<()> {
        println!("MockStorage: сохраняем {} трансферов", transfers.len());
        let mut locked_transfers = self.transfers.lock()
            .map_err(|e| anyhow::anyhow!("Ошибка блокировки мьютекса transfers: {}", e))
            .context("Failed to lock transfers mutex")?;
        *locked_transfers = transfers.to_vec();
        Ok(())
    }

    async fn save_stats(&self, stats: &[UserStats]) -> Result<()> {
        println!("MockStorage: сохраняем {} записей статистики", stats.len());
        let mut locked_stats = self.stats.lock()
            .map_err(|e| anyhow::anyhow!("Ошибка блокировки мьютекса stats: {}", e))
            .context("Failed to lock stats mutex")?;
        *locked_stats = stats.to_vec();
        locked_stats.sort_by(|a, b| b.total_volume.partial_cmp(&a.total_volume).unwrap_or(std::cmp::Ordering::Equal));
        Ok(())
    }

    async fn get_stats(&self) -> Result<Vec<UserStats>> {
        let stats = self.stats.lock()
            .map_err(|e| anyhow::anyhow!("Ошибка блокировки мьютекса stats: {}", e))
            .context("Failed to lock stats mutex for reading")?;
        println!("MockStorage: возвращаем {} записей статистики", stats.len());
        Ok(stats.clone())
    }
}
