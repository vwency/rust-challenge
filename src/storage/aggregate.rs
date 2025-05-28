use async_trait::async_trait;
use cqrs_es::{Aggregate, DomainEvent};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferEvent {
    TransferSaved {
        ts: u64,
        from: String,
        to: String,
        amount: f64,
        usd_price: f64,
    },
    UserStatsUpdated {
        address: String,
        total_volume: f64,
        avg_buy_price: f64,
        avg_sell_price: f64,
        max_balance: f64,
        max_balance_1h: f64,
        max_balance_24h: f64,
        max_balance_7d: f64,
    },
}

impl DomainEvent for TransferEvent {
    fn event_type(&self) -> String {
        match self {
            TransferEvent::TransferSaved { .. } => "TransferSaved".to_string(),
            TransferEvent::UserStatsUpdated { .. } => "UserStatsUpdated".to_string(),
        }
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferCommand {
    SaveTransfer {
        ts: u64,
        from: String,
        to: String,
        amount: f64,
        usd_price: f64,
    },
    UpdateUserStats {
        address: String,
        total_volume: f64,
        avg_buy_price: f64,
        avg_sell_price: f64,
        max_balance: f64,
        max_balance_1h: f64,
        max_balance_24h: f64,
        max_balance_7d: f64,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TransferAggregate {
    pub id: Uuid,
    pub transfers: Vec<TransferData>,
    pub user_stats: Option<UserStatsData>,
}

impl TransferAggregate {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            transfers: Vec::new(),
            user_stats: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransferData {
    pub ts: u64,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub usd_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserStatsData {
    pub address: String,
    pub total_volume: f64,
    pub avg_buy_price: f64,
    pub avg_sell_price: f64,
    pub max_balance: f64,
    pub max_balance_1h: f64,
    pub max_balance_24h: f64,
    pub max_balance_7d: f64,
}

#[derive(Debug, Error)]
pub enum TransferError {
    #[error("Command not implemented")]
    CommandNotImplemented,
    #[error("Invalid transfer data: {0}")]
    InvalidTransferData(String),
    #[error("Unknown error occurred")]
    UnknownError,
}

#[derive(Debug, Clone, Default)]
pub struct TransferServices;

// Собственный async трэйт для асинхронного handle
#[async_trait]
pub trait AsyncAggregate {
    type Command;
    type Event;
    type Error;
    type Services;

    async fn handle(
        &self,
        command: Self::Command,
        services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error>;

    fn apply(&mut self, event: Self::Event);
}

#[async_trait]
impl AsyncAggregate for TransferAggregate {
    type Command = TransferCommand;
    type Event = TransferEvent;
    type Error = TransferError;
    type Services = TransferServices;

    async fn handle(
        &self,
        command: Self::Command,
        _services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            TransferCommand::SaveTransfer { ts, from, to, amount, usd_price } => {
                if amount <= 0.0 {
                    return Err(TransferError::InvalidTransferData("Amount must be positive".into()));
                }
                Ok(vec![TransferEvent::TransferSaved { ts, from, to, amount, usd_price }])
            }
            TransferCommand::UpdateUserStats {
                address,
                total_volume,
                avg_buy_price,
                avg_sell_price,
                max_balance,
                max_balance_1h,
                max_balance_24h,
                max_balance_7d,
            } => Ok(vec![TransferEvent::UserStatsUpdated {
                address,
                total_volume,
                avg_buy_price,
                avg_sell_price,
                max_balance,
                max_balance_1h,
                max_balance_24h,
                max_balance_7d,
            }]),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            TransferEvent::TransferSaved { ts, from, to, amount, usd_price } => {
                self.transfers.push(TransferData {
                    ts,
                    from,
                    to,
                    amount,
                    usd_price,
                });
            }
            TransferEvent::UserStatsUpdated {
                address,
                total_volume,
                avg_buy_price,
                avg_sell_price,
                max_balance,
                max_balance_1h,
                max_balance_24h,
                max_balance_7d,
            } => {
                self.user_stats = Some(UserStatsData {
                    address,
                    total_volume,
                    avg_buy_price,
                    avg_sell_price,
                    max_balance,
                    max_balance_1h,
                    max_balance_24h,
                    max_balance_7d,
                });
            }
        }
    }
}
