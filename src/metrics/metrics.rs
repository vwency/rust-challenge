use crate::model::{Transfer, UserStats};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};

pub fn calculate_user_stats(transfers: &[Transfer]) -> Result<Vec<UserStats>> {
    let mut sorted_transfers = transfers.to_vec();
    sorted_transfers.sort_by_key(|t| t.ts);

    let mut balances: HashMap<String, f64> = HashMap::new();
    let mut max_balances: HashMap<String, f64> = HashMap::new();
    let mut max_balances_1h: HashMap<String, f64> = HashMap::new();
    let mut max_balances_24h: HashMap<String, f64> = HashMap::new();
    let mut max_balances_7d: HashMap<String, f64> = HashMap::new();
    let mut buy_prices: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
    let mut sell_prices: HashMap<String, Vec<(f64, f64)>> = HashMap::new();

    for t in &sorted_transfers {
        *balances.entry(t.from.clone()).or_default() -= t.amount;
        *balances.entry(t.to.clone()).or_default() += t.amount;

        let to_balance = balances.get(&t.to).copied().unwrap_or(0.0);
        let from_balance = balances.get(&t.from).copied().unwrap_or(0.0);

        max_balances
            .entry(t.to.clone())
            .and_modify(|b| *b = b.max(to_balance))
            .or_insert(to_balance.max(0.0));

        max_balances
            .entry(t.from.clone())
            .and_modify(|b| *b = b.max(from_balance))
            .or_insert(from_balance.max(0.0));

        buy_prices.entry(t.to.clone()).or_default().push((t.usd_price, t.amount));
        sell_prices.entry(t.from.clone()).or_default().push((t.usd_price, t.amount));
    }

    let mut balance_history: HashMap<String, Vec<(u64, f64)>> = HashMap::new();
    let mut current_balances: HashMap<String, f64> = HashMap::new();

    for t in &sorted_transfers {
        *current_balances.entry(t.from.clone()).or_default() -= t.amount;
        *current_balances.entry(t.to.clone()).or_default() += t.amount;

        balance_history
            .entry(t.from.clone())
            .or_default()
            .push((t.ts, current_balances[&t.from]));

        balance_history
            .entry(t.to.clone())
            .or_default()
            .push((t.ts, current_balances[&t.to]));
    }

    for (addr, history) in balance_history {
        let history_dt: Result<Vec<(DateTime<Utc>, f64)>> = history
            .into_iter()
            .map(|(ts, balance)| {
                DateTime::<Utc>::from_timestamp(ts as i64, 0)
                    .ok_or_else(|| anyhow::anyhow!("Invalid timestamp: {}", ts))
                    .context("Failed to convert timestamp to DateTime")
                    .map(|dt| (dt, balance))
            })
            .collect();

        let history_dt = history_dt.context("Failed to process balance history")?;

        if let Some((_, max_1h)) = calculate_max_balance_for_period(&history_dt, Duration::hours(1))
            .context("Failed to calculate max balance for 1 hour period")? {
            max_balances_1h.insert(addr.clone(), max_1h);
        }
        if let Some((_, max_24h)) = calculate_max_balance_for_period(&history_dt, Duration::hours(24))
            .context("Failed to calculate max balance for 24 hour period")? {
            max_balances_24h.insert(addr.clone(), max_24h);
        }
        if let Some((_, max_7d)) = calculate_max_balance_for_period(&history_dt, Duration::days(7))
            .context("Failed to calculate max balance for 7 day period")? {
            max_balances_7d.insert(addr.clone(), max_7d);
        }
    }

    let all_addresses: HashSet<_> = buy_prices.keys().chain(sell_prices.keys()).cloned().collect();

    let user_stats: Result<Vec<UserStats>> = all_addresses
        .into_iter()
        .map(|addr| {
            let buys = buy_prices.get(&addr).cloned().unwrap_or_default();
            let sells = sell_prices.get(&addr).cloned().unwrap_or_default();

            let total_volume: f64 = buys.iter().chain(&sells).map(|(_, amt)| amt).sum();

            let avg_weighted_price = |data: &[(f64, f64)]| -> Result<f64> {
                let (sum_weighted, sum_amount): (f64, f64) = data.iter().copied()
                    .fold((0.0, 0.0), |acc, (price, amount)| (acc.0 + price * amount, acc.1 + amount));

                if sum_amount > 0.0 {
                    if sum_weighted.is_finite() && sum_amount.is_finite() {
                        Ok(sum_weighted / sum_amount)
                    } else {
                        Err(anyhow::anyhow!("Invalid arithmetic result: sum_weighted={}, sum_amount={}", sum_weighted, sum_amount))
                            .context("Arithmetic overflow or invalid values in weighted average calculation")
                    }
                } else {
                    Ok(0.0)
                }
            };

            let avg_buy_price = avg_weighted_price(&buys)
                .context("Failed to calculate average buy price")?;
            let avg_sell_price = avg_weighted_price(&sells)
                .context("Failed to calculate average sell price")?;

            Ok(UserStats {
                address: addr.clone(),
                total_volume,
                avg_buy_price,
                avg_sell_price,
                max_balance: *max_balances.get(&addr).unwrap_or(&0.0),
                max_balance_1h: *max_balances_1h.get(&addr).unwrap_or(&0.0),
                max_balance_24h: *max_balances_24h.get(&addr).unwrap_or(&0.0),
                max_balance_7d: *max_balances_7d.get(&addr).unwrap_or(&0.0),
            })
        })
        .collect();

    user_stats.context("Failed to calculate user statistics")
}

fn calculate_max_balance_for_period(
    balance_history: &[(DateTime<Utc>, f64)],
    period: Duration,
) -> Result<Option<(DateTime<Utc>, f64)>> {
    let mut max_balance = 0.0;
    let mut max_timestamp = None;

    for (i, &(ts, balance)) in balance_history.iter().enumerate() {
        let window_end = ts.checked_add_signed(period)
            .ok_or_else(|| anyhow::anyhow!("DateTime arithmetic overflow when adding period to timestamp: {:?}", ts))
            .context("Failed to calculate window end time")?;

        let mut current_max = balance;

        for &(next_ts, next_balance) in &balance_history[i + 1..] {
            if next_ts > window_end {
                break;
            }
            if next_balance > current_max {
                current_max = next_balance;
            }
        }

        if current_max > max_balance {
            max_balance = current_max;
            max_timestamp = Some(ts);
        }
    }

    Ok(max_timestamp.map(|ts| (ts, max_balance)))
}
