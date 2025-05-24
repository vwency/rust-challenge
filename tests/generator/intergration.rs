use std::collections::HashSet;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use token_transfers::generator::{
    config::TransferGenConfig,
    transfer::{DefaultTransferGenerator, TransferGenerator, generate_transfers}
};

#[test]
fn test_large_scale_generation() {
    let transfers = generate_transfers(10000);
    assert_eq!(transfers.len(), 10000);

    // Spot check random transfers
    let config = TransferGenConfig::default();
    for i in [0, 100, 5000, 9999] {
        let transfer = &transfers[i];
        assert!(transfer.amount >= config.min_amount);
        assert!(transfer.amount <= config.max_amount);
        assert!(transfer.from.starts_with("0x"));
        assert!(transfer.to.starts_with("0x"));
    }
}

#[test]
fn test_thread_safety() {
    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                generate_transfers(100)
            })
        })
        .collect();

    let mut all_transfers = Vec::new();
    for handle in handles {
        let transfers = handle.join().unwrap();
        assert_eq!(transfers.len(), 100);
        all_transfers.extend(transfers);
    }

    assert_eq!(all_transfers.len(), 400);

    // Verify all transfers are valid
    for transfer in &all_transfers {
        assert!(transfer.from.starts_with("0x"));
        assert!(transfer.to.starts_with("0x"));
        assert!(transfer.amount > 0.0);
        assert!(transfer.usd_price > 0.0);
    }
}

#[test]
fn test_address_diversity() {
    let transfers = generate_transfers(100);
    let mut unique_addresses = HashSet::new();

    for transfer in &transfers {
        unique_addresses.insert(&transfer.from);
        unique_addresses.insert(&transfer.to);
    }

    // Should have good diversity (at least 150 unique addresses out of 200)
    assert!(unique_addresses.len() >= 150);
}

#[test]
fn test_end_to_end_workflow() {
    // Complete workflow test
    let config = TransferGenConfig {
        min_amount: 10.0,
        max_amount: 100.0,
        min_price: 0.5,
        max_price: 5.0,
        max_age_secs: 7200, // 2 hours
    };

    let generator = DefaultTransferGenerator::new(config.clone());
    let transfers = generator.generate(25);

    assert_eq!(transfers.len(), 25);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut unique_addresses = HashSet::new();

    for transfer in &transfers {
        // Value constraints
        assert!(transfer.amount >= config.min_amount);
        assert!(transfer.amount < config.max_amount);
        assert!(transfer.usd_price >= config.min_price);
        assert!(transfer.usd_price < config.max_price);

        // Time constraints
        assert!(transfer.ts <= now);
        assert!(transfer.ts >= now - config.max_age_secs);

        // Address constraints
        assert!(transfer.from.starts_with("0x"));
        assert!(transfer.to.starts_with("0x"));
        assert_eq!(transfer.from.len(), 42);
        assert_eq!(transfer.to.len(), 42);
        assert_ne!(transfer.from, transfer.to);

        unique_addresses.insert(&transfer.from);
        unique_addresses.insert(&transfer.to);
    }

    // Address diversity check
    assert!(unique_addresses.len() >= 40);
}

#[test]
fn test_multiple_generators_independence() {
    let config1 = TransferGenConfig::default();
    let config2 = TransferGenConfig {
        min_amount: 5.0,
        max_amount: 15.0,
        min_price: 1.0,
        max_price: 2.0,
        max_age_secs: 1800,
    };

    let gen1 = DefaultTransferGenerator::new(config1.clone());
    let gen2 = DefaultTransferGenerator::new(config2.clone());

    let transfers1 = gen1.generate(10);
    let transfers2 = gen2.generate(10);

    // Both should work independently
    assert_eq!(transfers1.len(), 10);
    assert_eq!(transfers2.len(), 10);

    // Check that they follow their respective configs
    for transfer in &transfers1 {
        assert!(transfer.amount >= config1.min_amount);
        assert!(transfer.amount <= config1.max_amount);
    }

    for transfer in &transfers2 {
        assert!(transfer.amount >= config2.min_amount);
        assert!(transfer.amount <= config2.max_amount);
    }
}