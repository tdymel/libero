//! Polling with a deadline. Never assert on a value you did not wait for: a bare read races dioxus.

use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use chromiumoxide::Page;

tokio::task_local! {
    static EXPECTING_FAILURE: u32;
}

/// `E2E_TIMEOUT_MS`, default 15 s: the right budget depends on machine load.
/// A share of it inside [`expecting_failure`].
pub(crate) fn timeout() -> Duration {
    let budget = std::env::var("E2E_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_secs(15));
    match EXPECTING_FAILURE.try_with(|share| *share) {
        Ok(share) => budget / share,
        Err(_) => budget,
    }
}

/// Runs a check meant to fail on a planted defect: a defect never heals, so the wait that
/// catches it need not run out the full budget (12 planted checks spent 15 s each on it).
pub async fn expecting_failure<F: std::future::Future>(check: F) -> F::Output {
    expecting_failure_in(3, check).await
}

/// The share a planted check tries first: 1 s of the default 15 s.
pub const QUICK_FAILURE_SHARE: u32 = 15;

/// [`expecting_failure`] with every wait at `1 / share` of the budget. A short share can
/// time out a setup step under load, so retry a wrong reason at the share of 3.
pub async fn expecting_failure_in<F: std::future::Future>(share: u32, check: F) -> F::Output {
    EXPECTING_FAILURE.scope(share.max(1), check).await
}

const POLL: Duration = Duration::from_millis(25);

/// The journal's name for an ordinary per-assertion poll (todo 364).
pub const READ: &str = "read";

/// Poll `check` until it returns true, or fail naming what was waited for.
pub async fn until<F, Fut>(what: &str, check: F) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    until_kind(READ, what, check).await
}

/// `until` under a journal kind. Reports poll count and slowest poll: many quick polls
/// mean a false condition, one long poll a stuck page or CDP connection (todo 364).
pub async fn until_kind<F, Fut>(kind: &'static str, what: &str, mut check: F) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    let budget = timeout();
    let started = Instant::now();
    let deadline = started + budget;
    let (mut polls, mut slowest) = (0u32, Duration::ZERO);
    loop {
        let poll_started = Instant::now();
        let outcome = check().await;
        polls += 1;
        slowest = slowest.max(poll_started.elapsed());

        match outcome {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(error) => {
                // A failed CDP call is how a dead connection looks; journal it too.
                crate::journal::gave_up(&crate::journal::GaveUp {
                    kind,
                    how: "failed",
                    what: &format!("{what} ({error})"),
                    budget,
                    elapsed: started.elapsed(),
                    slowest_poll: slowest,
                    polls,
                });
                return Err(error);
            }
        }

        if Instant::now() >= deadline {
            let elapsed = started.elapsed();
            crate::journal::gave_up(&crate::journal::GaveUp {
                kind,
                how: "expired",
                what,
                budget,
                elapsed,
                slowest_poll: slowest,
                polls,
            });
            bail!(
                "timed out after {elapsed:.1?} (budget {budget:.1?}) waiting for {what} \
                 [wait={kind}, {polls} poll(s), slowest {slowest:.2?}]"
            );
        }
        tokio::time::sleep(POLL).await;
    }
}

/// Waits for a JavaScript expression to become true. Use it after every interaction:
/// dioxus re-renders after the event returns.
pub async fn for_js_true(page: &Page, expression: &str, what: &str) -> Result<()> {
    until(what, || async {
        let value: bool = page.evaluate(expression).await?.into_value()?;
        Ok(value)
    })
    .await
}

/// Reads `expression`, runs `act`, then waits for it to change. A null before-value
/// fails, since a defaulted one passed on the first poll (todo 383).
///
/// ```ignore
/// wait::for_js_change(page, VALUE_NOW, "ArrowRight to step", || {
///     keyboard::press(page, keyboard::ARROW_RIGHT)
/// })
/// .await?;
/// ```
pub async fn for_js_change<A, Fut>(page: &Page, expression: &str, what: &str, act: A) -> Result<()>
where
    A: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let Some(previous) = read_string(page, expression).await? else {
        bail!(
            "waiting for {what}: `{expression}` reads nothing before it, so no change can be seen"
        );
    };
    act().await?;
    until(what, || {
        let previous = previous.clone();
        async move {
            let value = read_string(page, expression).await?;
            Ok(value.is_some_and(|v| v != previous))
        }
    })
    .await
}

/// `expression` as a string; `None` for `null`/`undefined`, which CDP sends as no value at
/// all, so `into_value::<Option<_>>` fails with "No value found" instead (1717).
async fn read_string(page: &Page, expression: &str) -> Result<Option<String>> {
    let result = page.evaluate(expression).await?;
    match result.value() {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(_) => Ok(Some(result.into_value()?)),
    }
}

/// Wait for an element to exist in the DOM.
pub async fn for_selector(page: &Page, selector: &str) -> Result<()> {
    for_selector_kind(READ, page, selector).await
}

/// `for_selector` under a journal kind, e.g. `fixture-ready` (todo 364).
pub async fn for_selector_kind(kind: &'static str, page: &Page, selector: &str) -> Result<()> {
    until_kind(kind, &format!("selector {selector}"), || {
        exists(page, selector)
    })
    .await
}

/// Whether `selector` matches right now. A negative check reads this on a state selector,
/// not `!is_visible`, which a card opened at opacity 0 passes (1719).
pub async fn exists(page: &Page, selector: &str) -> Result<bool> {
    Ok(page
        .evaluate(format!(
            "!!document.querySelector({})",
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?)
}

/// Waits for an element to be visible, not just present: a popover stays
/// `visibility: hidden` until measured (`codebase/use-popover`).
pub async fn for_visible(page: &Page, selector: &str) -> Result<()> {
    until(&format!("{selector} to be visible"), || {
        is_visible(page, selector)
    })
    .await
}

/// Whether `selector` is visible right now, by `for_visible`'s definition. Not the
/// negation of `for_hidden`: opacity 0 and an empty box count as neither.
pub async fn is_visible(page: &Page, selector: &str) -> Result<bool> {
    Ok(page
        .evaluate(format!(
            r#"(() => {{
                const el = document.querySelector({});
                if (!el) return false;
                const s = getComputedStyle(el);
                if (s.visibility === 'hidden' || s.display === 'none' || s.opacity === '0') return false;
                const r = el.getBoundingClientRect();
                return r.width > 0 && r.height > 0;
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?)
}

/// Wait for an element to be gone from the DOM, or present but invisible.
pub async fn for_hidden(page: &Page, selector: &str) -> Result<()> {
    until(&format!("{selector} to be hidden"), || async {
        let hidden: bool = page
            .evaluate(format!(
                r#"(() => {{
                    const el = document.querySelector({});
                    if (!el) return true;
                    const s = getComputedStyle(el);
                    return s.visibility === 'hidden' || s.display === 'none';
                }})()"#,
                serde_json::to_string(selector)?
            ))
            .await?
            .into_value()?;
        Ok(hidden)
    })
    .await
}
