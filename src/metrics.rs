use crate::model::{Transfer, UserStats};
use std::collections::HashMap;

pub fn calculate_user_stats(transfers: &[Transfer]) -> Vec<UserStats> {
    let mut sorted_transfers = transfers.to_vec();
    sorted_transfers.sort_by_key(|t| t.ts);

    let mut balances: HashMap<String, f64> = HashMap::new();
    let mut max_balances: HashMap<String, f64> = HashMap::new();
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

        buy_prices
            .entry(t.to.clone())
            .or_default()
            .push((t.usd_price, t.amount));

        sell_prices
            .entry(t.from.clone())
            .or_default()
            .push((t.usd_price, t.amount));
    }

    let all_addresses: std::collections::HashSet<_> = buy_prices
        .keys()
        .chain(sell_prices.keys())
        .cloned()
        .collect();

    all_addresses
        .into_iter()
        .map(|addr| {
            let buys = buy_prices.get(&addr).cloned().unwrap_or_default();
            let sells = sell_prices.get(&addr).cloned().unwrap_or_default();

            let total_volume: f64 = buys.iter().chain(&sells).map(|(_, amt)| amt).sum();

            let avg_weighted_price = |data: &[(f64, f64)]| {
                let (sum_weighted, sum_amount): (f64, f64) = data
                    .iter()
                    .copied()
                    .fold((0.0, 0.0), |acc, (price, amount)| {
                        (acc.0 + price * amount, acc.1 + amount)
                    });

                if sum_amount > 0.0 {
                    sum_weighted / sum_amount
                } else {
                    0.0
                }
            };

            UserStats {
                address: addr.clone(),
                total_volume,
                avg_buy_price: avg_weighted_price(&buys),
                avg_sell_price: avg_weighted_price(&sells),
                max_balance: *max_balances.get(&addr).unwrap_or(&0.0),
            }
        })
        .collect()
}
