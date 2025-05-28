use async_trait::async_trait;
use uuid::Uuid;

use crate::storage::errors::StorageError;
use crate::storage::events::TransferEvent;

// Интерфейс event store
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append_event(&self, aggregate_id: Uuid, event: TransferEvent) -> Result<(), StorageError>;
}

// Пример реализации event store с ClickHouse
pub struct ClickHouseEventStore {
    client: clickhouse::Client,
}

impl ClickHouseEventStore {
    pub async fn new(client: clickhouse::Client) -> Result<Self, StorageError> {
        // Инициализация таблицы событий
        client.query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                aggregate_id String,
                event_type String,
                payload String,
                timestamp UInt64
            ) ENGINE = MergeTree() ORDER BY timestamp
            "#
        ).execute().await.map_err(StorageError::ClickHouse)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl EventStore for ClickHouseEventStore {
    async fn append_event(&self, aggregate_id: Uuid, event: TransferEvent) -> Result<(), StorageError> {
        let event_type = match &event {
            TransferEvent::TransferSaved { .. } => "TransferSaved",
            TransferEvent::UserStatsUpdated { .. } => "UserStatsUpdated",
        };

        let payload = serde_json::to_string(&event).map_err(|e| StorageError::EventStore(e.to_string()))?;

        self.client
            .query("INSERT INTO events (aggregate_id, event_type, payload, timestamp) VALUES (?, ?, ?, ?)")
            .bind(aggregate_id.to_string())
            .bind(event_type)
            .bind(payload)
            .bind(chrono::Utc::now().timestamp() as u64)
            .execute()
            .await
            .map_err(StorageError::ClickHouse)?;

        Ok(())
    }
}
