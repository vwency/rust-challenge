mod model;
mod generator;
mod storage;
mod metrics;

use storage::{ClickHouseStorage, MockStorage, Storage};
use crate::metrics::calculate_user_stats;
use generator::generate_transfers;
use std::env;
use std::sync::Arc;
use clickhouse::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Сервис анализа трансферов токенов ===\n");

    println!("Генерируем тестовые данные...");
    let transfers = generate_transfers(10000)?;
    println!("Сгенерировано {} трансферов", transfers.len());

    let use_mock = env::var("USE_MOCK").unwrap_or_else(|_| "false".to_string()) == "true";

    let storage: Arc<dyn Storage> = if use_mock {
        println!("\nИспользуем мок-хранилище...");
        Arc::new(MockStorage::new())
    } else {
        println!("\nПытаемся подключиться к ClickHouse...");
        let client = Client::default().with_url("http://clickhouse:8123");

        match ClickHouseStorage::new(client).await {
            Ok(storage) => {
                println!("Подключение к ClickHouse успешно!");
                Arc::new(storage)
            }
            Err(e) => {
                println!("Ошибка подключения к ClickHouse: {}", e);
                println!("Переходим на мок-хранилище...");
                Arc::new(MockStorage::new())
            }
        }
    };

    run_with_storage(storage, &transfers).await?;

    Ok(())
}

async fn run_with_storage(
    storage: Arc<dyn Storage>,
    transfers: &[model::Transfer],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Сохраняем трансферы...");
    storage.save_transfers(transfers).await?;

    println!("Рассчитываем метрики...");
    let stats = calculate_user_stats(transfers)?;
    println!("Рассчитано метрик для {} адресов", stats.len());

    println!("Сохраняем статистику...");
    storage.save_stats(&stats).await?;

    println!("Читаем статистику из хранилища...");
    let saved_stats = storage.get_stats().await?;

    println!("\n=== ТОП-10 адресов по объему ===");
    for (i, stat) in saved_stats.iter().take(10).enumerate() {
        println!("{}. Адрес: {}", i + 1, &stat.address[..10]);
        println!("   Общий объем: {:.2}", stat.total_volume);
        println!("   Средняя цена покупки: {:.4}", stat.avg_buy_price);
        println!("   Средняя цена продажи: {:.4}", stat.avg_sell_price);
        println!("   Максимальный баланс: {:.2}\n", stat.max_balance);
    }

    println!("Всего обработано адресов: {}", saved_stats.len());

    let total_volume: f64 = saved_stats.iter().map(|s| s.total_volume).sum();
    let avg_volume: f64 = if !saved_stats.is_empty() {
        total_volume / saved_stats.len() as f64
    } else {
        0.0
    };

    println!("\n=== ОБЩАЯ СТАТИСТИКА ===");
    println!("Общий объем всех операций: {:.2}", total_volume);
    println!("Средний объем на адрес: {:.2}", avg_volume);

    Ok(())
}
