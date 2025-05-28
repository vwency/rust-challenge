#[cfg(test)]
mod tests {
    use chrono::{Utc, TimeZone};
    use mycrate::pipeline::calculate_user_stats;
    use mycrate::model::{Transfer};
    use anyhow::Context;

    #[test]
    fn test_empty_transfers() -> anyhow::Result<()> {
        let transfers: Vec<Transfer> = vec![];
        let stats = calculate_user_stats(&transfers).context("Failed to calculate user stats")?;
        assert!(stats.is_empty());
        Ok(())
    }

    #[test]
    fn test_single_transfer() -> anyhow::Result<()> {
        let ts = Utc.with_ymd_and_hms(2025, 5, 25, 12, 0, 0)
            .single()
            .context("Invalid timestamp")?
            .timestamp() as u64;

        let transfers = vec![
            Transfer {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 10.0,
                ts,
                usd_price: 5.0,
            }
        ];

        let stats = calculate_user_stats(&transfers).context("Failed to calculate user stats")?;

        let alice_stats = stats.iter()
            .find(|s| s.address == "Alice")
            .context("Alice stats not found")?;
        let bob_stats = stats.iter()
            .find(|s| s.address == "Bob")
            .context("Bob stats not found")?;

        assert!(alice_stats.max_balance >= 0.0);
        assert_eq!(alice_stats.total_volume, 10.0);
        assert_eq!(alice_stats.avg_sell_price, 5.0);
        assert_eq!(alice_stats.avg_buy_price, 0.0);

        assert!(bob_stats.max_balance >= 0.0);
        assert_eq!(bob_stats.total_volume, 10.0);
        assert_eq!(bob_stats.avg_buy_price, 5.0);
        assert_eq!(bob_stats.avg_sell_price, 0.0);

        Ok(())
    }

    #[test]
    fn test_multiple_transfers() -> anyhow::Result<()> {
        let base_ts = Utc.with_ymd_and_hms(2025, 5, 25, 12, 0, 0)
            .single()
            .context("Invalid timestamp")?
            .timestamp() as u64;

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

        let stats = calculate_user_stats(&transfers).context("Failed to calculate user stats")?;

        for stat in &stats {
            assert!(stat.total_volume > 0.0);
        }

        let bob_stats = stats.iter()
            .find(|s| s.address == "Bob")
            .context("Bob stats not found")?;
        assert_eq!(bob_stats.avg_buy_price, 5.0);
        assert_eq!(bob_stats.avg_sell_price, 6.0);

        let charlie_stats = stats.iter()
            .find(|s| s.address == "Charlie")
            .context("Charlie stats not found")?;
        assert!(charlie_stats.max_balance >= 0.0);

        Ok(())
    }

    #[test]
    fn test_max_balance_calculation() -> anyhow::Result<()> {
        let base_ts = Utc.with_ymd_and_hms(2025, 5, 25, 12, 0, 0)
            .single()
            .context("Invalid timestamp")?
            .timestamp() as u64;

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

        let stats = calculate_user_stats(&transfers).context("Failed to calculate user stats")?;

        let a_stats = stats.iter()
            .find(|s| s.address == "A")
            .context("A stats not found")?;

        assert!(a_stats.max_balance >= 0.0);

        Ok(())
    }
}
