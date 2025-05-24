use crate::model::Transfer;
use rand::{distributions::Alphanumeric, Rng};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::generator::config::TransferGenConfig;

pub trait TransferGenerator {
    fn generate(&self, count: usize) -> Vec<Transfer>;
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
    fn generate(&self, count: usize) -> Vec<Transfer> {
        let mut rng = rand::thread_rng();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        (0..count)
            .map(|_| {
                let from = rand_address(&mut rng);
                let to = rand_address(&mut rng);
                let amount = rng.gen_range(self.config.min_amount..self.config.max_amount);
                let usd_price = rng.gen_range(self.config.min_price..self.config.max_price);
                let ts = now - rng.gen_range(0..self.config.max_age_secs);

                Transfer {
                    ts,
                    from,
                    to,
                    amount,
                    usd_price,
                }
            })
            .collect()
    }
}

fn rand_address(rng: &mut impl Rng) -> String {
    let suffix: String = rng
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    format!("0x{}", suffix)
}

pub fn generate_transfers(count: usize) -> Vec<Transfer> {
    let config = TransferGenConfig::default();
    let generator = DefaultTransferGenerator::new(config);
    generator.generate(count)
}
