#[cfg(test)]
mod tests {
    use chrono::{Utc, TimeZone};
    use crate::metrics::calculate_user_stats;
    use crate::model::{Transfer, UserStats};

    #[test]
    fn test_empty_transfers() {
        let transfers: Vec<Transfer> = vec![];
        let stats = calculate_user_stats(&transfers);
        assert!(stats.is_empty());
    }

    #[test]
    fn test_single_transfer() {
        let ts = Utc.ymd(2025, 5, 25).and_hms(12, 0, 0).timestamp() as u64;

        let transfers = vec![
            Transfer {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 10.0,
                ts,
                usd_price: 5.0,
            }
        ];

        let stats = calculate_user_stats(&transfers);

        let alice_stats = stats.iter().find(|s| s.address == "Alice").unwrap();
        let bob_stats = stats.iter().find(|s| s.address == "Bob").unwrap();

        assert!(alice_stats.max_balance >= 0.0);
        assert_eq!(alice_stats.total_volume, 10.0);
        assert_eq!(alice_stats.avg_sell_price, 5.0);
        assert_eq!(alice_stats.avg_buy_price, 0.0);

        assert!(bob_stats.max_balance >= 0.0);
        assert_eq!(bob_stats.total_volume, 10.0);
        assert_eq!(bob_stats.avg_buy_price, 5.0);
        assert_eq!(bob_stats.avg_sell_price, 0.0);
    }

    #[test]
    fn test_multiple_transfers() {
        let base_ts = Utc.ymd(2025, 5, 25).and_hms(12, 0, 0).timestamp() as u64;

        let transfers = vec![
            Transfer {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 10.0,
                ts: base_ts,
                usd_price: 5.0,
            },
            Transfer {
                from: "Bob".to_string(),
                to: "Charlie".to_string(),
                amount: 5.0,
                ts: base_ts + 3600,
                usd_price: 6.0,
            },
            Transfer {
                from: "Alice".to_string(),
                to: "Charlie".to_string(),
                amount: 2.0,
                ts: base_ts + 7200,
                usd_price: 4.0,
            },
        ];

        let stats = calculate_user_stats(&transfers);

        for stat in &stats {
            assert!(stat.total_volume > 0.0);
        }

        let bob_stats = stats.iter().find(|s| s.address == "Bob").unwrap();
        assert_eq!(bob_stats.avg_buy_price, 5.0);
        assert_eq!(bob_stats.avg_sell_price, 6.0);

        let charlie_stats = stats.iter().find(|s| s.address == "Charlie").unwrap();
        assert!(charlie_stats.max_balance >= 0.0);
    }

    #[test]
    fn test_max_balance_calculation() {
        let base_ts = Utc.ymd(2025, 5, 25).and_hms(12, 0, 0).timestamp() as u64;

        let transfers = vec![
            Transfer {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 10.0,
                ts: base_ts,
                usd_price: 1.0,
            },
            Transfer {
                from: "B".to_string(),
                to: "A".to_string(),
                amount: 5.0,
                ts: base_ts + 1800,
                usd_price: 1.0,
            },
            Transfer {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 3.0,
                ts: base_ts + 3600,
                usd_price: 1.0,
            },
        ];

        let stats = calculate_user_stats(&transfers);

        let a_stats = stats.iter().find(|s| s.address == "A").unwrap();

        assert!(a_stats.max_balance_1h >= 0.0);
        assert!(a_stats.max_balance_24h >= a_stats.max_balance_1h);
    }
}
