//! Polling with a deadline: the auto-waiting we give up by not using
//! Playwright.
//!
//! Every read against the app goes through one of these. A bare read races the
//! app's mount, and a race is what produced most of the false readings recorded
//! in `codebase/testing`. The rule is boring and absolute: never assert on a
//! value you did not wait for.

use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use chromiumoxide::Page;

/// Overridable, because the right value depends on the machine and on how much
/// else is running. A fixed budget that is fine on an idle laptop is the
/// classic source of "passes locally, flakes under load".
fn timeout() -> Duration {
    std::env::var("E2E_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_secs(15))
}

const POLL: Duration = Duration::from_millis(25);

/// The journal's name for an ordinary per-assertion poll, the one
/// `E2E_TIMEOUT_MS` bounds. `Fixture::open` passes its own names, because a
/// red run has to say which of the three waits gave up (todo 364).
pub const READ: &str = "read";

/// Poll `check` until it returns true, or fail naming what was waited for.
pub async fn until<F, Fut>(what: &str, check: F) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    until_kind(READ, what, check).await
}

/// `until`, with the journal's name for this kind of wait.
///
/// Besides polling it measures, because "it took 15 s" and "one call took
/// 120 s" are the two mechanisms behind todo 364 and the wall-clock number
/// alone cannot tell them apart. The failure message and the journal line
/// both carry the poll count and the slowest single poll: hundreds of quick
/// polls mean the condition never came true, one long poll means the page or
/// the CDP connection stopped answering.
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
                // The poll itself failed - a CDP call that errored or timed
                // out. Worth a journal line of its own: it is the shape a
                // dead browser connection takes, and it never reaches the
                // `expired` branch below.
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

/// Wait for a JavaScript expression to become true.
///
/// **Use this instead of reading a value straight after an interaction.** A
/// keypress or a click returns as soon as the event is dispatched; dioxus then
/// runs the handler, writes a signal, and re-renders on its own schedule. An
/// assertion that reads the DOM on the next line is racing that, and it is the
/// single largest source of flakiness in a browser suite - the kind that passes
/// on an idle machine and fails when twelve tests share one.
pub async fn for_js_true(page: &Page, expression: &str, what: &str) -> Result<()> {
    until(what, || async {
        let value: bool = page.evaluate(expression).await?.into_value()?;
        Ok(value)
    })
    .await
}

/// Wait for a JavaScript expression to stop equalling `previous`.
///
/// The shape wanted after an interaction that should change something: "the
/// value moved", without hard-coding what it moved to.
pub async fn for_js_change(
    page: &Page,
    expression: &str,
    previous: &str,
    what: &str,
) -> Result<()> {
    let previous = previous.to_string();
    until(what, || {
        let previous = previous.clone();
        async move {
            let value: Option<String> = page.evaluate(expression).await?.into_value()?;
            Ok(value.is_some_and(|v| v != previous))
        }
    })
    .await
}

/// Wait for an element to exist in the DOM.
pub async fn for_selector(page: &Page, selector: &str) -> Result<()> {
    for_selector_kind(READ, page, selector).await
}

/// `for_selector`, under a journal name of the caller's choosing.
/// `Fixture::open` uses it so the `[data-fixture-ready]` wait is not filed as
/// an ordinary read (todo 364).
pub async fn for_selector_kind(kind: &'static str, page: &Page, selector: &str) -> Result<()> {
    until_kind(kind, &format!("selector {selector}"), || async {
        let found: bool = page
            .evaluate(format!(
                "!!document.querySelector({})",
                serde_json::to_string(selector)?
            ))
            .await?
            .into_value()?;
        Ok(found)
    })
    .await
}

/// Wait for an element to be **visible**, not merely present.
///
/// This is the distinction the popover work paid for once: a popover is laid
/// out `visibility: hidden` until its first measurement lands, because
/// `display: none` has nothing to measure. Waiting on presence therefore
/// returns while the box is still invisible and unfocusable - and
/// `focus()` on a `visibility: hidden` element does nothing *and reports
/// success* (`codebase/use-popover`). So the suite waits on placed, never on
/// mounted.
pub async fn for_visible(page: &Page, selector: &str) -> Result<()> {
    until(&format!("{selector} to be visible"), || {
        is_visible(page, selector)
    })
    .await
}

/// Whether `selector` is visible right now, by `for_visible`'s definition.
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
