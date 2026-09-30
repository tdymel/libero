//! A clock the test holds, for the timers it names and no others.

use anyhow::{Result, bail};
use chromiumoxide::Page;

/// Install with `(HELD_CLOCK)([ms, ...])`; timeouts and intervals with those delays wait for
/// `window.__heldClock.armed(ms)` / `.fire(ms)` / `.fireAll(ms)`, all others run on the real clock.
/// Not virtual time: that stops rAF and dioxus effects too, so the arm/fire order is lost.
pub const HELD_CLOCK: &str = r#"(delays) => {
    if (window.__heldClock) return;
    const realSet = window.setTimeout.bind(window);
    const realClear = window.clearTimeout.bind(window);
    const realEvery = window.setInterval.bind(window);
    const realClearEvery = window.clearInterval.bind(window);
    const held = new Map();
    let next = -1;
    const hold = (fn, ms, args, repeats) => {
        const id = next--;
        held.set(id, { ms, repeats, run: () => fn(...args) });
        return id;
    };
    const run = ([id, t]) => {
        if (!t.repeats) held.delete(id);
        t.run();
    };
    window.__heldClock = {
        armed: (ms) => [...held.values()].filter((t) => t.ms === ms).length,
        fire: (ms) => {
            const hits = [...held].filter(([, t]) => t.ms === ms);
            if (hits.length !== 1) return hits.length;
            run(hits[0]);
            return 1;
        },
        fireAll: (ms) => {
            const hits = [...held].filter(([, t]) => t.ms === ms);
            hits.forEach(run);
            return hits.length;
        },
    };
    window.setTimeout = function (fn, ms, ...args) {
        if (delays.includes(ms) && typeof fn === 'function') return hold(fn, ms, args, false);
        return realSet(fn, ms, ...args);
    };
    window.setInterval = function (fn, ms, ...args) {
        if (delays.includes(ms) && typeof fn === 'function') return hold(fn, ms, args, true);
        return realEvery(fn, ms, ...args);
    };
    window.clearTimeout = function (id) {
        if (!held.delete(id)) realClear(id);
    };
    window.clearInterval = function (id) {
        if (!held.delete(id)) realClearEvery(id);
    };
}"#;

/// Holds the page's timers of `delays` from now on; ones armed before keep the real clock.
pub async fn hold(page: &Page, delays: &[u32]) -> Result<()> {
    page.evaluate(format!(
        "(({HELD_CLOCK})({}), true)",
        serde_json::to_string(delays)?
    ))
    .await?;
    Ok(())
}

/// Holds the timers of `delays` from the page's load on, those set at mount too: reloads.
pub async fn hold_from_load(page: &Page, delays: &[u32]) -> Result<()> {
    page.evaluate_on_new_document(format!(
        "({HELD_CLOCK})({})",
        serde_json::to_string(delays)?
    ))
    .await?;
    page.reload().await?;
    Ok(())
}

/// How many held timers of `ms` are pending.
pub async fn armed(page: &Page, ms: u32) -> Result<usize> {
    Ok(page
        .evaluate(format!("window.__heldClock.armed({ms})"))
        .await?
        .into_value()?)
}

/// Waits until `count` held timers of `ms` are pending.
pub async fn until_armed(page: &Page, ms: u32, count: usize, what: &str) -> Result<()> {
    crate::wait::for_js_true(
        page,
        &format!("window.__heldClock.armed({ms}) === {count}"),
        what,
    )
    .await
}

/// Fires the one pending timer of `ms`; fails unless exactly one is pending.
pub async fn fire(page: &Page, ms: u32) -> Result<()> {
    let fired: usize = page
        .evaluate(format!("window.__heldClock.fire({ms})"))
        .await?
        .into_value()?;
    if fired != 1 {
        bail!("{fired} held timers of {ms} ms pending, expected one to fire");
    }
    Ok(())
}

/// Returns after two macrotask turns of the page: what a handler or a fired timer queued
/// (dioxus renders on microtasks) has run. The signal a "nothing happened" check reads
/// after, in place of a sleep. No frame: a background tab throttles rAF to 500 ms.
pub async fn settle(page: &Page) -> Result<()> {
    page.evaluate("new Promise((done) => setTimeout(() => setTimeout(() => done(true), 0), 0))")
        .await?;
    Ok(())
}

/// Returns once the page drew its next frame. Up to a second in a tab not in front.
pub async fn next_frame(page: &Page) -> Result<()> {
    page.evaluate("new Promise((done) => requestAnimationFrame(() => done(true)))")
        .await?;
    Ok(())
}

/// [`next_frame`], then [`settle`]: what that frame queued (an observer's report, a scroll
/// event) has run.
pub async fn frame(page: &Page) -> Result<()> {
    next_frame(page).await?;
    settle(page).await
}

/// Fires every pending timer of `ms` (an interval stays armed); returns how many ran.
pub async fn fire_all(page: &Page, ms: u32) -> Result<usize> {
    Ok(page
        .evaluate(format!("window.__heldClock.fireAll({ms})"))
        .await?
        .into_value()?)
}
