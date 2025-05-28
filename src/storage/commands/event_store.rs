use async_trait::async_trait;
use crate::storage::events::Event;

#[async_trait]
pub trait EventStore {
    async fn append_events(&self, aggregate_id: &str, events: &[Event]) -> Result<(), StorageError>;
    async fn load_events(&self, aggregate_id: &str) -> Result<Vec<Event>, StorageError>;
}

pub struct ClickHouseEventStore {
    client: Client,
}

impl ClickHouseEventStore {
    pub async fn new(client: Client) -> Result<Self, StorageError> {
        // Создать таблицу events, если еще нет
        client.query(r#"
            CREATE TABLE IF NOT EXISTS events (
                aggregate_id String,
                event_type String,
                payload String,
                timestamp UInt64
            ) ENGINE = MergeTree() ORDER BY (aggregate_id, timestamp)
        "#).execute().await.map_err(StorageError::ClickHouse)?;

        Ok(Self { client })
    }
}

#[async_trait]
impl EventStore for ClickHouseEventStore {
    async fn append_events(&self, aggregate_id: &str, events: &[Event]) -> Result<(), StorageError> {
        // сериализуем события и вставляем в таблицу events
        // ...
        Ok(())
    }

    async fn load_events(&self, aggregate_id: &str) -> Result<Vec<Event>, StorageError> {
        // читаем события по aggregate_id и десериализуем
        // ...
        Ok(vec![])
    }
}
