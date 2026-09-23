//! Frame time on a page: an injected recorder (rAF deltas, long animation frames) and the
//! stats read off it. Report only; headless numbers are machine-specific, compare ratios.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

/// The first frames carry the interaction's own start-up.
const WARM_UP_FRAMES: usize = 10;

const RECORDER: &str = r#"(() => {
    window.__frames?.stop();
    const frames = { deltas: [], long: 0, running: true, stop() {} };
    let last = null;
    const tick = (now) => {
        if (!frames.running) return;
        if (last !== null) frames.deltas.push(now - last);
        last = now;
        requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    let observer = null;
    try {
        observer = new PerformanceObserver((list) => { frames.long += list.getEntries().length; });
        observer.observe({ type: 'long-animation-frame' });
    } catch (_) {}
    frames.stop = () => { frames.running = false; observer?.disconnect(); };
    window.__frames = frames;
})()"#;

/// Frame deltas in milliseconds, summarised.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrameStats {
    pub p50: f64,
    pub p95: f64,
    pub max: f64,
    pub count: usize,
    /// Frames the browser reported as long animation frames (50 ms and over).
    pub long: usize,
}

impl FrameStats {
    /// `deltas` in milliseconds, the warm-up frames dropped.
    pub fn from_deltas(deltas: &[f64], long: usize) -> Self {
        let mut sorted: Vec<f64> = deltas.iter().skip(WARM_UP_FRAMES).copied().collect();
        sorted.sort_by(f64::total_cmp);
        Self {
            p50: percentile(&sorted, 0.50),
            p95: percentile(&sorted, 0.95),
            max: sorted.last().copied().unwrap_or(0.0),
            count: sorted.len(),
            long,
        }
    }
}

/// Nearest rank over an ascending slice; 0 when empty.
fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (fraction * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

/// Starts recording; a recorder already on the page is stopped and replaced. The tab is
/// brought to front first: in a background tab every input event waits out a 500 ms frame.
pub async fn start(page: &Page) -> Result<()> {
    page.bring_to_front()
        .await
        .context("bring the page to front")?;
    page.evaluate(RECORDER)
        .await
        .context("inject the frame recorder")?;
    Ok(())
}

/// Stops the recorder and returns what it saw.
pub async fn stop(page: &Page) -> Result<FrameStats> {
    let json: String = page
        .evaluate(
            "(() => { const f = window.__frames; f.stop(); \
             return JSON.stringify({ deltas: f.deltas, long: f.long }); })()",
        )
        .await
        .context("read the frame recorder; was `frames::start` called?")?
        .into_value()?;
    #[derive(Deserialize)]
    struct Recorded {
        deltas: Vec<f64>,
        long: usize,
    }
    let recorded: Recorded = serde_json::from_str(&json)?;
    Ok(FrameStats::from_deltas(&recorded.deltas, recorded.long))
}

/// Prints one table row and merges `stats` under `name` into `frame-time.json` in the target dir.
pub fn report(name: &str, stats: FrameStats) -> Result<()> {
    println!(
        "{name:<28} frames {:>4}  p50 {:>6.1} ms  p95 {:>6.1} ms  max {:>6.1} ms  long {:>3}",
        stats.count, stats.p50, stats.p95, stats.max, stats.long
    );
    let path = json_path()?;
    let mut all: BTreeMap<String, FrameStats> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    all.insert(name.to_string(), stats);
    std::fs::write(&path, serde_json::to_string_pretty(&all)?)
        .with_context(|| format!("write {}", path.display()))
}

/// `<target dir>/frame-time.json`, the target dir this test binary was built into.
fn json_path() -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    // <target>/<profile>/deps/<binary>
    let target = exe
        .ancestors()
        .nth(3)
        .context("the test binary sits in a target dir")?;
    Ok(target.join("frame-time.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_drop_the_warm_up_and_rank_the_rest() {
        let mut deltas = vec![500.0; WARM_UP_FRAMES];
        deltas.extend((1..=100).map(f64::from));
        let stats = FrameStats::from_deltas(&deltas, 2);
        assert_eq!(stats.count, 100);
        assert_eq!(stats.p50, 50.0);
        assert_eq!(stats.p95, 95.0);
        assert_eq!(stats.max, 100.0);
        assert_eq!(stats.long, 2);
    }

    #[test]
    fn no_frames_is_all_zero() {
        let stats = FrameStats::from_deltas(&[], 0);
        assert_eq!((stats.count, stats.max, stats.p50), (0, 0.0, 0.0));
    }
}
