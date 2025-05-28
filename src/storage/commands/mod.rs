pub mod save_stats;
pub mod save_transfers;

// Если нужно, можно здесь переэкспортировать удобные типы:

pub use save_stats::{SaveStatsCommand, ClickHouseSaveStatsCommand};
pub use save_transfers::{SaveTransfersCommand, ClickHouseSaveTransfersCommand};
