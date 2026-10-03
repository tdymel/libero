//! Main-thread time from Chromium's CDP `Performance` counters, shared by the timing
//! reports (`perf::timing`, `table_perf`, `tests/docs_perf.rs`). Enable the domain first.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::performance::GetMetricsParams;

/// `name -> seconds` for the counters a rep reads.
pub async fn metrics(page: &Page) -> Result<BTreeMap<String, f64>> {
    let reply = page.execute(GetMetricsParams::default()).await?;
    Ok(reply
        .result
        .metrics
        .iter()
        .map(|metric| (metric.name.clone(), metric.value))
        .collect())
}

/// `key`'s growth from `before` to `after`, in milliseconds.
pub fn delta_ms(after: &BTreeMap<String, f64>, before: &BTreeMap<String, f64>, key: &str) -> f64 {
    (after.get(key).copied().unwrap_or(0.0) - before.get(key).copied().unwrap_or(0.0)) * 1e3
}

/// Until 60 ms pass with under 2 ms of main-thread work: effects, placement passes and
/// exit animations belong to the interaction that started them.
pub async fn quiet(page: &Page) -> Result<()> {
    let started = Instant::now();
    let mut last = metrics(page).await?;
    loop {
        tokio::time::sleep(Duration::from_millis(60)).await;
        let now = metrics(page).await?;
        if delta_ms(&now, &last, "TaskDuration") < 2.0 {
            return Ok(());
        }
        // A 5000-row Table mount works for seconds.
        if started.elapsed() > Duration::from_secs(20) {
            bail!("the page never went quiet");
        }
        last = now;
    }
}
