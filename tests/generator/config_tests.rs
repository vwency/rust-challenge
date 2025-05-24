use token_transfers::generator::config::TransferGenConfig;

#[test]
fn test_default_config() {
    let config = TransferGenConfig::default();

    assert_eq!(config.min_amount, 1.0);
    assert_eq!(config.max_amount, 1000.0);
    assert_eq!(config.min_price, 0.1);
    assert_eq!(config.max_price, 2.0);
    assert_eq!(config.max_age_secs, 86_400 * 30); // 30 days
}

#[test]
fn test_config_clone() {
    let config1 = TransferGenConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.min_amount, config2.min_amount);
    assert_eq!(config1.max_amount, config2.max_amount);
    assert_eq!(config1.min_price, config2.min_price);
    assert_eq!(config1.max_price, config2.max_price);
    assert_eq!(config1.max_age_secs, config2.max_age_secs);
}

#[test]
fn test_custom_config() {
    let config = TransferGenConfig {
        min_amount: 5.0,
        max_amount: 50.0,
        min_price: 1.0,
        max_price: 3.0,
        max_age_secs: 3600,
    };

    assert_eq!(config.min_amount, 5.0);
    assert_eq!(config.max_amount, 50.0);
    assert_eq!(config.min_price, 1.0);
    assert_eq!(config.max_price, 3.0);
    assert_eq!(config.max_age_secs, 3600);
}
