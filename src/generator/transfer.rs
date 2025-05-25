use anyhow::{Context, Result};
use rand::Rng;
use rand::prelude::ThreadRng;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::Transfer;
use crate::generator::config::TransferGenConfig;
use crate::generator::address::rand_address;

pub trait TransferGenerator {
    fn generate(&self, count: usize) -> Result<Vec<Transfer>>;
}

pub struct DefaultTransferGenerator {
    pub config: TransferGenConfig,
}

impl DefaultTransferGenerator {
    pub fn new(config: TransferGenConfig) -> Self {
        Self { config }
    }
}

impl TransferGenerator for DefaultTransferGenerator {
    fn generate(&self, count: usize) -> Result<Vec<Transfer>> {
        let mut rng: ThreadRng = rand::thread_rng();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get duration since UNIX_EPOCH")?
            .as_secs();

        let transfers = (0..count)
            .map(|_| {
                let from = rand_address(&mut rng);
                let to = rand_address(&mut rng);
                let amount = rng.gen_range(self.config.min_amount..self.config.max_amount);
                let usd_price = rng.gen_range(self.config.min_price..self.config.max_price);
                let ts = if self.config.max_age_secs == 0 {
                    now
                } else {
                    now - rng.gen_range(0..self.config.max_age_secs)
                };

                Transfer {
                    ts,
                    from,
                    to,
                    amount,
                    usd_price,
                }
            })
            .collect();

        Ok(transfers)
    }
}

pub fn generate_transfers(count: usize) -> Result<Vec<Transfer>> {
    let config = TransferGenConfig::default();
    let generator = DefaultTransferGenerator::new(config);
    generator.generate(count)
}
