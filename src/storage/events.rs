use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferEvent {
    /// Событие сохранения перевода.
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
