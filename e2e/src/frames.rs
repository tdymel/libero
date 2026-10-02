//! Frame time on a page: an injected recorder (rAF deltas, long animation frames) and the
//! stats read off it. Report only; headless numbers are machine-specific, compare ratios.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

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

/// Activating a tab exits fullscreen in the others: a test in fullscreen holds this shared
/// ([`keep_fullscreen`]), [`bring_to_front`] takes it alone.
static FOREGROUND: tokio::sync::RwLock<()> = tokio::sync::RwLock::const_new(());

tokio::task_local! {
    /// Set while this test holds [`keep_fullscreen`]: its own gestures skip the lock.
    static HOLDS_FULLSCREEN: Cell<bool>;
}

/// Runs a test body where [`keep_fullscreen`] can mark its holder; `browser::block_on`
/// wraps every test in it.
pub(crate) async fn scoped<F: Future>(body: F) -> F::Output {
    HOLDS_FULLSCREEN.scope(Cell::new(false), body).await
}

/// This test's share of [`FOREGROUND`], from entering fullscreen to the test's end.
#[must_use = "fullscreen lasts as long as this is held"]
pub struct Fullscreen {
    _shared: tokio::sync::RwLockReadGuard<'static, ()>,
}

impl Drop for Fullscreen {
    fn drop(&mut self) {
        let _ = HOLDS_FULLSCREEN.try_with(|holds| holds.set(false));
    }
}

/// Held from entering fullscreen to the test's end, no other tab is brought to front.
pub async fn keep_fullscreen() -> Fullscreen {
    let shared = FOREGROUND.read().await;
    let _ = HOLDS_FULLSCREEN.try_with(|holds| holds.set(true));
    Fullscreen { _shared: shared }
}

/// A page in front; no test enters fullscreen until [`Front::release`].
#[must_use = "release it before closing the page"]
pub struct Front {
    _alone: tokio::sync::RwLockWriteGuard<'static, ()>,
}

impl Front {
    /// Brings the harness's home tab to front; call it before closing the page.
    pub async fn release(self) -> Result<()> {
        crate::browser::home()
            .await?
            .bring_to_front()
            .await
            .context("bring the home tab to front")?;
        Ok(())
    }
}

/// Brings `page` to front once no test holds [`keep_fullscreen`], within the wait budget
/// (todo 1755); a wait past 100 ms goes to the journal.
pub async fn bring_to_front(page: &Page) -> Result<Front> {
    let started = Instant::now();
    let alone = tokio::time::timeout(crate::wait::timeout(), FOREGROUND.write())
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "waited {:.1}s to bring a page to front: a test in fullscreen holds the tabs",
                started.elapsed().as_secs_f64()
            )
        })?;
    let waited = started.elapsed();
    if waited > Duration::from_millis(100) {
        crate::journal::note(&format!(
            "front: waited {:.1}s for the fullscreen tests",
            waited.as_secs_f64()
        ));
    }
    page.bring_to_front()
        .await
        .context("bring the page to front")?;
    Ok(Front { _alone: alone })
}

/// Runs `work` with `page` in front: a tab behind it draws about a frame a second, and each
/// pointer move, wheel or touch event waits for one. Pages open behind the home tab.
/// Inside [`keep_fullscreen`] the work runs where it is: the lock is this test's own, and
/// bringing a tab to front would end the other tests' fullscreen.
pub async fn in_front<T>(page: &Page, work: impl Future<Output = Result<T>>) -> Result<T> {
    if HOLDS_FULLSCREEN.try_with(Cell::get).unwrap_or(false) {
        return work.await;
    }
    let front = bring_to_front(page).await?;
    let done = work.await;
    // Released either way; the work's own error comes first.
    let released = front.release().await;
    let done = done?;
    released?;
    Ok(done)
}

/// Starts recording; a recorder already on the page is stopped and replaced. The tab is
/// brought to front first: in a background tab every input event waits out a 500 ms frame.
pub async fn start(page: &Page) -> Result<Front> {
    let front = bring_to_front(page).await?;
    page.evaluate(RECORDER)
        .await
        .context("inject the frame recorder")?;
    Ok(front)
}

/// Waits until the recorder holds `count` frames past the warm-up.
pub async fn until_recorded(page: &Page, count: usize) -> Result<()> {
    crate::wait::for_js_true(
        page,
        &format!(
            "window.__frames.deltas.length >= {}",
            WARM_UP_FRAMES + count
        ),
        &format!("{count} recorded frames"),
    )
    .await
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
