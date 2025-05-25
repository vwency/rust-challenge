use crate::model::{Transfer, UserStats};
use crate::storage::{Storage, errors::StorageError};
use async_trait::async_trait;
use std::sync::Mutex;

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
    async fn save_transfers(&self, transfers: &[Transfer]) -> Result<(), StorageError> {
        println!("MockStorage: сохраняем {} трансферов", transfers.len());
        let mut locked_transfers = self.transfers.lock()
            .map_err(|e| StorageError::Generic(format!("Ошибка блокировки мьютекса transfers: {}", e)))?;
        *locked_transfers = transfers.to_vec();
        Ok(())
    }

    async fn save_stats(&self, stats: &[UserStats]) -> Result<(), StorageError> {
        println!("MockStorage: сохраняем {} записей статистики", stats.len());
        let mut locked_stats = self.stats.lock()
            .map_err(|e| StorageError::Generic(format!("Ошибка блокировки мьютекса stats: {}", e)))?;
        *locked_stats = stats.to_vec();
        locked_stats.sort_by(|a, b| b.total_volume.partial_cmp(&a.total_volume).unwrap_or(std::cmp::Ordering::Equal));
        Ok(())
    }

    async fn get_stats(&self) -> Result<Vec<UserStats>, StorageError> {
        let stats = self.stats.lock()
            .map_err(|e| StorageError::Generic(format!("Ошибка блокировки мьютекса stats: {}", e)))?;
        println!("MockStorage: возвращаем {} записей статистики", stats.len());
        Ok(stats.clone())
    }
}
