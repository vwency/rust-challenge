use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub ts: u64,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub usd_price: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub address: String,
    pub total_volume: f64,
    pub avg_buy_price: f64,
    pub avg_sell_price: f64,
    pub max_balance: f64,
    pub max_balance_1h: f64,    // Максимальный баланс за 1 час
    pub max_balance_24h: f64,   // Максимальный баланс за 24 часа
    pub max_balance_7d: f64,    // Максимальный баланс за 7 дней
}
