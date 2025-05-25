use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

use mycrate::generator::{
    config::TransferGenConfig,
    transfer::{DefaultTransferGenerator, TransferGenerator, generate_transfers}
};

#[test]
fn test_generate_transfers_count() -> Result<()> {
    let transfers = generate_transfers(10)?;
    assert_eq!(transfers.len(), 10);

    let transfers = generate_transfers(0)?;
    assert_eq!(transfers.len(), 0);

    let transfers = generate_transfers(100)?;
    assert_eq!(transfers.len(), 100);

    Ok(())
}

#[test]
fn test_transfer_generator_basic() -> Result<()> {
    let config = TransferGenConfig::default();
    let generator = DefaultTransferGenerator::new(config.clone());
    let transfers = generator.generate(20)?;

    assert_eq!(transfers.len(), 20);

    for transfer in &transfers {
        assert!(transfer.amount >= config.min_amount);
        assert!(transfer.amount <= config.max_amount);

        assert!(transfer.usd_price >= config.min_price);
        assert!(transfer.usd_price <= config.max_price);

        assert!(transfer.from.starts_with("0x"));
        assert!(transfer.to.starts_with("0x"));
        assert_eq!(transfer.from.len(), 42);
        assert_eq!(transfer.to.len(), 42);
        assert_ne!(transfer.from, transfer.to);

        assert!(transfer.amount > 0.0);
        assert!(transfer.usd_price > 0.0);
        assert!(transfer.amount.is_finite());
        assert!(transfer.usd_price.is_finite());
    }

    Ok(())
}

#[test]
fn test_transfer_timestamps() -> Result<()> {
    let config = TransferGenConfig::default();
    let generator = DefaultTransferGenerator::new(config.clone());
    let transfers = generator.generate(10)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    for transfer in &transfers {
        assert!(transfer.ts <= now);
        assert!(transfer.ts >= now - config.max_age_secs);
        assert!(transfer.ts > 0);
    }

    Ok(())
}

#[test]
fn test_custom_config_ranges() -> Result<()> {
    let config = TransferGenConfig {
        min_amount: 5.0,
        max_amount: 50.0,
        min_price: 1.0,
        max_price: 3.0,
        max_age_secs: 3600,
    };

    let generator = DefaultTransferGenerator::new(config.clone());
    let transfers = generator.generate(30)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    for transfer in &transfers {
        assert!(transfer.amount >= config.min_amount);
        assert!(transfer.amount <= config.max_amount);
        assert!(transfer.usd_price >= config.min_price);
        assert!(transfer.usd_price <= config.max_price);

        assert!(transfer.ts >= now - config.max_age_secs);
    }

    Ok(())
}

#[test]
fn test_edge_case_small_ranges() -> Result<()> {
    let config = TransferGenConfig {
        min_amount: 1.0,
        max_amount: 1.1,
        min_price: 0.5,
        max_price: 0.6,
        max_age_secs: 1,
    };

    let generator = DefaultTransferGenerator::new(config.clone());
    let transfers = generator.generate(5)?;

    for transfer in &transfers {
        assert!(transfer.amount >= config.min_amount);
        assert!(transfer.amount <= config.max_amount);
        assert!(transfer.usd_price >= config.min_price);
        assert!(transfer.usd_price <= config.max_price);
    }

    Ok(())
}

#[test]
fn test_zero_max_age() -> Result<()> {
    let config = TransferGenConfig {
        min_amount: 1.0,
        max_amount: 10.0,
        min_price: 0.1,
        max_price: 1.0,
        max_age_secs: 0,
    };

    let generator = DefaultTransferGenerator::new(config);
    let transfers = generator.generate(5)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    for transfer in &transfers {
        assert!(transfer.ts >= now - 1);
        assert!(transfer.ts <= now);
    }

    Ok(())
}
