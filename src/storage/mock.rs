use crate::model::{Transfer, UserStats};
use crate::storage::{Storage, StorageError};
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
