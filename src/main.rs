mod model;
mod generator;
mod storage;
mod pipeline;

use std::env;
use std::sync::Arc;

use clickhouse::Client;

use crate::generator::generate_transfers;
use crate::pipeline::calculate_user_stats;
use crate::storage::{ClickHouseStorage, MockStorage, Storage};

const DEFAULT_TRANSFERS_COUNT: usize = 10_000;
const CLICKHOUSE_URL: &str = "http://clickhouse:8123";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let transfers = generate_test_data(DEFAULT_TRANSFERS_COUNT)?;
    let storage = initialize_storage().await;

    run_analysis(storage, &transfers).await?;

    Ok(())
}

fn print_header() {
    println!("=== Сервис анализа трансферов токенов ===\n");
}

fn generate_test_data(count: usize) -> Result<Vec<model::Transfer>, Box<dyn std::error::Error>> {
    println!("Генерируем тестовые данные...");

    let transfers = generate_transfers(count)?;
    println!("Сгенерировано {} трансферов\n", transfers.len());

    Ok(transfers)
}

async fn initialize_storage() -> Arc<dyn Storage> {
    let use_mock = should_use_mock_storage();

    if use_mock {
        println!("Использование мок-хранилища...");
        return Arc::new(MockStorage::new());
    }

    println!("Попытка подключения к ClickHouse...");

    match create_clickhouse_storage().await {
        Ok(storage) => {
            println!("✓ Подключение к ClickHouse успешно!\n");
            Arc::new(storage)
        }
        Err(error) => {
            println!("✗ Ошибка подключения к ClickHouse: {}", error);
            println!("Переход на мок-хранилище...\n");
            Arc::new(MockStorage::new())
        }
    }
}

fn should_use_mock_storage() -> bool {
    env::var("USE_MOCK")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true"
}

async fn create_clickhouse_storage() -> Result<ClickHouseStorage, Box<dyn std::error::Error>> {
    let client = Client::default().with_url(CLICKHOUSE_URL);
    ClickHouseStorage::new(client).await.map_err(|e| e.into())
}

async fn run_analysis(
    storage: Arc<dyn Storage>,
    transfers: &[model::Transfer],
) -> Result<(), Box<dyn std::error::Error>> {
    save_transfers(&storage, transfers).await?;
    
    let _stats = calculate_and_save_statistics(&storage, transfers).await?;
    
    let _saved_stats = storage.get_stats().await?;
    println!("Анализ завершен успешно!");

    Ok(())
}

async fn save_transfers(
    storage: &Arc<dyn Storage>,
    transfers: &[model::Transfer],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Сохранение трансферов...");
    storage.save_transfers(transfers).await?;
    println!("✓ Трансферы сохранены");

    Ok(())
}

async fn calculate_and_save_statistics(
    storage: &Arc<dyn Storage>,
    transfers: &[model::Transfer],
) -> Result<Vec<crate::model::UserStats>, Box<dyn std::error::Error>> {
    println!("Расчет метрик...");
    let stats = calculate_user_stats(transfers)?;
    println!("✓ Рассчитано метрик для {} адресов", stats.len());

    println!("Сохранение статистики...");
    storage.save_stats(&stats).await?;
    println!("✓ Статистика сохранена\n");

    Ok(stats)
}
