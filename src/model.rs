use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct Transfer {
    pub ts: u64,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub usd_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct UserStats {
    pub address: String,
    pub total_volume: f64,
    pub avg_buy_price: f64,
    pub avg_sell_price: f64,
    pub max_balance: f64,
    pub max_balance_1h: f64,
    pub max_balance_24h: f64,
    pub max_balance_7d: f64,
}
