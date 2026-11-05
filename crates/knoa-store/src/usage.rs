//! AI usage, added up per day and model.

use knoa_core::{ModelUsage, Usage, UsageTotals};
use rusqlite::params;

use crate::{Result, Store};

impl Store {
    /// Adds requests to `day`'s totals. `day` is a local date, `YYYY-MM-DD`.
    pub fn record_usage(&mut self, day: &str, provider: &str, usage: &[Usage]) -> Result<()> {
        if usage.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO ai_usage (day, provider, model, requests, input_tokens, output_tokens,
                                       cost, unmetered, unpriced)
                 VALUES (?1, ?2, ?3, 1, coalesce(?4, 0), coalesce(?5, 0), coalesce(?6, 0),
                         ?4 IS NULL OR ?5 IS NULL, ?6 IS NULL)
                 ON CONFLICT (day, provider, model) DO UPDATE SET
                     requests = requests + 1,
                     input_tokens = input_tokens + excluded.input_tokens,
                     output_tokens = output_tokens + excluded.output_tokens,
                     cost = cost + excluded.cost,
                     unmetered = unmetered + excluded.unmetered,
                     unpriced = unpriced + excluded.unpriced",
            )?;
            for request in usage {
                stmt.execute(params![
                    day,
                    provider,
                    request.model,
                    request.input_tokens.map(saturating_i64),
                    request.output_tokens.map(saturating_i64),
                    request.cost,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Each model's totals from `since` on (a local date, `YYYY-MM-DD`), or ever. Most
    /// expensive first, then most used.
    pub fn usage_by_model(&self, since: Option<&str>) -> Result<Vec<ModelUsage>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT provider, model, sum(requests), sum(input_tokens), sum(output_tokens),
                    sum(cost), sum(unmetered), sum(unpriced)
             FROM ai_usage WHERE ?1 IS NULL OR day >= ?1
             GROUP BY provider, model
             ORDER BY sum(cost) DESC, sum(requests) DESC",
        )?;
        let rows = stmt.query_map([since], |row| {
            Ok(ModelUsage {
                provider: row.get(0)?,
                model: row.get(1)?,
                totals: UsageTotals {
                    requests: row.get(2)?,
                    input_tokens: row.get(3)?,
                    output_tokens: row.get(4)?,
                    cost: row.get(5)?,
                    unmetered: row.get(6)?,
                    unpriced: row.get(7)?,
                },
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

/// SQLite integers are signed; no real token count comes near the limit.
fn saturating_i64(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(model: &str, tokens: Option<(u64, u64)>, cost: Option<f64>) -> Usage {
        Usage {
            model: model.into(),
            input_tokens: tokens.map(|t| t.0),
            output_tokens: tokens.map(|t| t.1),
            cost,
        }
    }

    #[test]
    fn totals_add_up_and_count_what_the_provider_left_out() {
        let mut store = Store::open_in_memory().unwrap();
        store
            .record_usage(
                "2026-11-04",
                "openrouter",
                &[request("a", Some((100, 10)), Some(0.001))],
            )
            .unwrap();
        store
            .record_usage(
                "2026-11-05",
                "openrouter",
                &[
                    request("a", Some((200, 20)), None),
                    request("a", None, Some(0.002)),
                ],
            )
            .unwrap();

        let all = store.usage_by_model(None).unwrap();
        assert_eq!(all.len(), 1);
        let totals = all[0].totals;
        assert_eq!(
            (totals.requests, totals.input_tokens, totals.output_tokens),
            (3, 300, 30)
        );
        assert!((totals.cost - 0.003).abs() < 1e-12);
        assert_eq!((totals.unmetered, totals.unpriced), (1, 1));

        let today = store.usage_by_model(Some("2026-11-05")).unwrap();
        assert_eq!(today[0].totals.requests, 2);
    }
}
