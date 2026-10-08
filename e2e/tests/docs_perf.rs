//! The docs timing survey (todo 2104): main-thread cost of mounting every docs page, of the
//! same markup set as plain DOM, and of the controls found on each page. A report, not a gate:
//! `cargo test -p e2e --test docs_perf -- --ignored --nocapture survey` with `E2E_BASE_URL` on a
//! `dx serve --release` docs server (`cargo run -p e2e -- docs-perf` serves a debug build). Rounds
//! run the whole page list again (A B A B); round 0 is the first visit, the summary takes the later rounds.
//!
//! - `DOCS_PERF_PAGES`: comma-separated parts of page paths, unset takes every nav page.
//! - `DOCS_PERF_KINDS`: comma-separated row kinds (`mount`, `plain`, `theme`, `scroll`,
//!   `toggle`, `segment`, `select`, `type`, `key`, `drag`, `tab`, `popup`, `button`), unset takes all.
//! - `DOCS_PERF_ROUNDS` (default 4), `DOCS_PERF_PER_KIND` controls per kind and page (2).
//! - `DOCS_PERF_OUT`: the JSON report, default `<target>/docs-perf.json`.
//! - `DOCS_PERF_B`: a second docs server (another build). Each round then runs A and B in turn
//!   (A B, B A, ...), each in its own tab, so a warm row stays warm as in a single-variant run;
//!   the report adds B - A per row. On a loaded machine compare `low`, the
//!   mean of the fastest third, over 8 rounds or more; the report names both builds and the load.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType,
};
use chromiumoxide::cdp::browser_protocol::performance::EnableParams;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, ARROW_LEFT, ARROW_RIGHT, BACKSPACE, ESCAPE};
use e2e::passes::pointer::{self, Point};
use e2e::timing::{delta_ms, metrics, quiet};
use e2e::{Fixture, Scheme, Viewport, frames};
use serde::{Deserialize, Serialize};

/// Rendered by the docs shell once a route's page is in it.
const READY: &str = "#docs-main *";

/// rAF gaps, long animation frames and every DOM write, reset per rep.
const RECORDER: &str = r#"(() => {
    if (window.__dp) return;
    const t = { frames: [], loaf: 0, writes: 0, last: null, dead: null };
    addEventListener('error', (e) => { t.dead = t.dead || String(e.message); });
    const tick = (now) => {
        if (t.last !== null) t.frames.push(now - t.last);
        t.last = now;
        requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    try {
        new PerformanceObserver((l) => { t.loaf += l.getEntries().length; })
            .observe({ type: 'long-animation-frame' });
    } catch (_) {}
    new MutationObserver((records) => {
        for (const r of records) t.writes += r.type === 'childList'
            ? r.addedNodes.length + r.removedNodes.length : 1;
    }).observe(document.documentElement,
        { childList: true, subtree: true, attributes: true, characterData: true });
    t.reset = () => { t.frames = []; t.loaf = 0; t.writes = 0; };
    t.read = () => JSON.stringify({ frame: Math.max(0, ...t.frames), loaf: t.loaf, writes: t.writes,
        nodes: document.querySelectorAll('#docs-main *').length });
    window.__dp = t;
})()"#;

/// `describe(el)`: tag, role, slot and a short name; the sweep's, kept apart so either can move.
const DESCRIBE: &str = r#"const describe = (el) => {
    let s = el.tagName.toLowerCase();
    const role = el.getAttribute('role');
    if (role) s += `[role=${role}]`;
    const name = (el.getAttribute('aria-label') || el.textContent || el.value
        || el.getAttribute('placeholder') || '').trim().replace(/\s+/g, ' ').slice(0, 28);
    return name ? `${s} "${name}"` : s;
};"#;

/// Each kind's selector inside `#docs-main`, first match wins, so a switch is no `button`.
const KINDS: &[(&str, &str)] = &[
    (
        "toggle",
        "[role=switch], input[type=checkbox], [role=checkbox], [role=radio]:not([data-slot=segment] *)",
    ),
    ("segment", "[data-slot=segment]"),
    ("select", "select"),
    (
        "type",
        "input:not([type]), input[type=text], input[type=search], input[type=email], \
         input[type=number], textarea, [contenteditable=true]",
    ),
    ("key", "[role=slider], [role=spinbutton]"),
    ("drag", "[role=slider], [role=separator][tabindex]"),
    ("tab", "[role=tab]"),
    (
        "popup",
        "[aria-haspopup]:not([aria-haspopup=false]), [role=combobox]",
    ),
    ("button", "button:not([role])"),
];

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct Rep {
    task: f64,
    script: f64,
    /// Style recalculation and layout.
    layout: f64,
    /// The longest rAF gap; 0.0 means the recorder never ticked, the rep is invalid.
    frame: f64,
    long: u32,
    /// DOM nodes added or removed plus attribute and text writes.
    writes: u32,
    /// Elements in `#docs-main` afterwards.
    nodes: u32,
}

#[derive(Deserialize)]
struct Seen {
    frame: f64,
    loaf: u32,
    writes: u32,
    nodes: u32,
}

#[derive(Default, Serialize)]
struct Row {
    /// The first visit's rep, round 0.
    first: Vec<Rep>,
    /// Every later round's reps.
    warm: Vec<Rep>,
    invalid: u32,
    failed: Vec<String>,
}

type Rows = BTreeMap<String, Row>;

/// The docs server the running variant is measured on.
static BASE: Mutex<String> = Mutex::new(String::new());

fn base() -> String {
    BASE.lock().unwrap().clone()
}

/// One build under test: `A` is `E2E_BASE_URL`, `B` is `DOCS_PERF_B`.
struct Variant {
    label: &'static str,
    base: String,
    /// The served wasm file name: its hash tells two builds apart.
    build: String,
    rows: Rows,
}

/// Machine load and duration of one variant's round, kept with the numbers it explains.
#[derive(Serialize)]
struct RoundMeta {
    round: usize,
    variant: &'static str,
    load_before: f64,
    load_after: f64,
    secs: f64,
}

/// The 1-minute load average; NaN where `/proc` is missing.
fn load_average() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}

fn env_list(name: &str) -> Option<Vec<String>> {
    std::env::var(name)
        .ok()
        .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn wanted(kinds: &Option<Vec<String>>, kind: &str) -> bool {
    kinds.as_ref().is_none_or(|k| k.iter().any(|k| k == kind))
}

async fn js<T: serde::de::DeserializeOwned>(page: &Page, expression: &str) -> Result<T> {
    Ok(page.evaluate(expression).await?.into_value()?)
}

/// Polls `condition` for up to 1.5 s: a control that does not react fails its row, not the run.
async fn until(page: &Page, condition: &str, what: &str) -> Result<()> {
    let started = Instant::now();
    while started.elapsed() < Duration::from_millis(1500) {
        if js::<bool>(page, condition).await.unwrap_or(false) {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    bail!("{what}: {condition} stayed false")
}

/// One measured rep of `act`, from a quiet page to a quiet page.
async fn rep<F: AsyncFnOnce() -> Result<()>>(page: &Page, act: F) -> Result<Rep> {
    // Headless Chromium runs no rAF in a tab that lost the front, and some acts take it.
    page.bring_to_front().await?;
    quiet(page).await?;
    page.evaluate("window.__dp.reset()").await?;
    let before = metrics(page).await?;
    act().await?;
    quiet(page).await?;
    let after = metrics(page).await?;
    let seen: Seen = serde_json::from_str(&js::<String>(page, "window.__dp.read()").await?)?;
    Ok(Rep {
        task: delta_ms(&after, &before, "TaskDuration"),
        script: delta_ms(&after, &before, "ScriptDuration"),
        layout: delta_ms(&after, &before, "LayoutDuration")
            + delta_ms(&after, &before, "RecalcStyleDuration"),
        frame: seen.frame,
        long: seen.loaf,
        writes: seen.writes,
        nodes: seen.nodes,
    })
}

fn record(rows: &mut Rows, name: String, round: usize, outcome: Result<Rep>) {
    let row = rows.entry(name).or_default();
    match outcome {
        Ok(r) if r.frame == 0.0 => row.invalid += 1,
        Ok(r) if round == 0 => row.first.push(r),
        Ok(r) => row.warm.push(r),
        Err(e) => row.failed.push(format!("{e:#}")),
    }
}

/// Every collapsed nav section opened, so each page has a link to click.
async fn expand_nav(page: &Page) -> Result<()> {
    const COLLAPSED: &str = "#docs-nav [role=treeitem][aria-expanded=false]";
    for _ in 0..60 {
        let before: usize = js(
            page,
            &format!("document.querySelectorAll({COLLAPSED:?}).length"),
        )
        .await?;
        if before == 0 {
            return Ok(());
        }
        page.evaluate(format!(
            "document.querySelector({COLLAPSED:?}).querySelector('[data-tree-chevron], button, a, span')?.click() \
             ?? document.querySelector({COLLAPSED:?}).click()"
        ))
        .await?;
        until(
            page,
            &format!("document.querySelectorAll({COLLAPSED:?}).length < {before}"),
            "a nav section to expand",
        )
        .await?;
    }
    Ok(())
}

/// Where [`aim`] tags the nav link to click.
const NAV_LINK: &str = "[data-docs-perf-nav]";

/// Tags `path`'s nav link and scrolls it into view, before a timed [`navigate`], which
/// then writes nothing; `false` when the nav has no link to it.
async fn aim(page: &Page, path: &str) -> Result<bool> {
    let p = serde_json::to_string(path)?;
    js(
        page,
        &format!(
            "(() => {{ const a = [...document.querySelectorAll('#docs-nav a[href]')] \
               .find((a) => new URL(a.href).pathname === {p}); \
               if (!a) return false; \
               if (a.hasAttribute('data-docs-perf-nav')) return true; \
               document.querySelectorAll('{NAV_LINK}').forEach((a) => a.removeAttribute('data-docs-perf-nav')); \
               a.setAttribute('data-docs-perf-nav', ''); a.scrollIntoView({{ block: 'nearest' }}); return true; }})()"
        ),
    )
    .await
}

/// An in-app navigation to `path`: its nav link clicked by pointer, which moves the focus
/// as a user's click does (todo 2324), else a router `popstate`.
async fn navigate(page: &Page, path: &str) -> Result<()> {
    let p = serde_json::to_string(path)?;
    match aim(page, path).await? {
        true => pointer::click(page, NAV_LINK).await?,
        false => {
            page.evaluate(format!(
                "history.pushState(null, '', {p}); dispatchEvent(new PopStateEvent('popstate'))"
            ))
            .await?;
        }
    }
    until(
        page,
        &format!(
            "location.pathname === {p} && document.querySelector('#docs-main h1, #docs-main h2') !== null"
        ),
        "the page to mount",
    )
    .await
}

/// [`navigate`], else a full load of `path`: a control can leave the shell unable to route
/// (a theme set, an open overlay). Says so, as that is a finding too.
async fn ensure(page: &Page, path: &str) -> Result<()> {
    if let Err(e) = navigate(page, path).await {
        let at: String = js(page, "location.pathname").await.unwrap_or_default();
        eprintln!("docs-perf: in-app navigation {at} -> {path} failed ({e:#}), reloading");
        load(page, path).await?;
    }
    Ok(())
}

/// A full load of `path` from the current variant's server, recorder and nav set up again.
async fn load(page: &Page, path: &str) -> Result<()> {
    page.goto(format!("{}{path}", base())).await?;
    until(
        page,
        "document.querySelector('#docs-main *') !== null",
        "the reload",
    )
    .await?;
    page.evaluate(RECORDER).await?;
    expand_nav(page).await
}

/// `#docs-main`'s markup set again as plain DOM beside it: the lean baseline of a mount.
async fn plain(page: &Page) -> Result<()> {
    page.evaluate(
        "(() => { const m = document.querySelector('#docs-main'); window.__dpHtml = m.innerHTML; \
         const d = document.createElement(m.tagName); d.className = m.className; d.id = 'docs-perf-plain'; \
         m.style.display = 'none'; m.before(d); d.innerHTML = window.__dpHtml; })()",
    )
    .await?;
    Ok(())
}

async fn unplain(page: &Page) -> Result<()> {
    page.evaluate(
        "(() => { document.getElementById('docs-perf-plain')?.remove(); \
         document.querySelector('#docs-main').style.display = ''; })()",
    )
    .await?;
    Ok(())
}

/// A full load of `path` when the app threw (a wasm panic leaves every control dead).
async fn revive(page: &Page, path: &str, after: &str) -> Result<()> {
    let error: String = js(
        page,
        "window.__dp ? (window.__dp.dead || '') : 'no recorder'",
    )
    .await?;
    if !error.is_empty() {
        eprintln!("docs-perf: the app threw after {after}: {error}; reloading {path}");
        load(page, path).await?;
    }
    Ok(())
}

/// The theme control in the docs header.
const THEME: &str = "header button[data-slot=toggle]";

/// The toggle names the scheme it switches to.
const SCHEME: &str =
    "document.querySelector('header button[data-slot=toggle]').getAttribute('aria-label')";

/// The element the docs page scrolls in.
const SCROLLER: &str = "(() => { for (let el = document.querySelector('#docs-main'); el; el = el.parentElement) \
     { const s = getComputedStyle(el).overflowY; if ((s === 'auto' || s === 'scroll') && el.scrollHeight > el.clientHeight + 1) return el; } \
     return document.scrollingElement; })()";

#[derive(Deserialize, Debug)]
struct Control {
    kind: String,
    id: u32,
    /// `demo` (a demo's preview), `controls` (its prop panel) or `page`.
    zone: String,
    name: String,
}

/// Tags up to `per_kind` controls of each kind and zone, in document order: `data-dp-<id>` on
/// the control, `data-dp-hit-<id>` on what a pointer hits (a visible ancestor of a hidden input).
async fn controls(page: &Page, per_kind: usize) -> Result<Vec<Control>> {
    let kinds = serde_json::to_string(KINDS)?;
    js(
        page,
        &format!(
            "(() => {{ {DESCRIBE} const main = document.querySelector('#docs-main'); \
             const safe = (e) => e && getComputedStyle(e).justifyContent.includes('safe'); \
             const zone = (el) => {{ for (let a = el; a && a !== main; a = a.parentElement) {{ \
               if (safe(a)) return 'demo'; if (safe(a.previousElementSibling)) return 'controls'; }} return 'page'; }}; \
             const big = (e) => {{ const r = e.getBoundingClientRect(); return r.width >= 8 && r.height >= 8; }}; \
             for (const e of main.querySelectorAll('*')) for (const a of [...e.attributes]) \
               if (a.name.startsWith('data-dp-')) e.removeAttribute(a.name); \
             const out = []; const taken = new Set(); const count = {{}}; let id = 0; \
             for (const [kind, sel] of {kinds}) {{ \
               for (const el of main.querySelectorAll(sel)) {{ \
                 if (kind !== 'drag' && taken.has(el)) continue; \
                 if (!el.getClientRects().length || el.closest('pre, code, a[href], [aria-disabled=true], :disabled, [inert], [aria-hidden=true]')) continue; \
                 let hit = el; while (hit && hit !== main && !big(hit)) hit = hit.parentElement; \
                 if (!hit || hit === main || getComputedStyle(hit).visibility === 'hidden') continue; \
                 if (kind === 'button' && /copy|code/i.test(el.getAttribute('aria-label') || el.textContent)) continue; \
                 const z = zone(el); const key = kind + z; count[key] = count[key] || 0; \
                 if (count[key] >= {per_kind}) continue; \
                 taken.add(el); el.setAttribute('data-dp-' + id, ''); hit.setAttribute('data-dp-hit-' + id, ''); \
                 out.push({{ kind, id, zone: z, name: describe(el) }}); id++; count[key]++; }} }} \
             return out; }})()"
        ),
    )
    .await
}

/// Clicks `item` of the group around `el` (`items` matched inside `group`), `selected`
/// telling the picked one, then the one picked before: the rows `select` and `back`.
#[allow(clippy::too_many_arguments)]
async fn pick_and_back(
    page: &Page,
    el: &str,
    c: &Control,
    group: &str,
    items: &str,
    selected: &str,
    round: usize,
    rows: &mut Rows,
    name: impl Fn(&str) -> String,
) {
    let Ok((was, to)) = js::<(i64, i64)>(
        page,
        &format!(
            "(() => {{ const items = [...{el}.closest({group:?}).querySelectorAll({items:?})]; \
             items.forEach((t, i) => t.setAttribute('data-dp-item', '{id}-' + i)); \
             const was = items.findIndex((t) => {selected}); \
             const mine = items.indexOf({el}); \
             return [was, mine === was ? (mine + 1) % items.length : mine]; }})()",
            id = c.id
        ),
    )
    .await
    else {
        return;
    };
    if was < 0 || was == to {
        return;
    }
    for (step, index) in [("select", to), ("back", was)] {
        let target = format!("[data-dp-item='{}-{index}']", c.id);
        let t = serde_json::to_string(&target).unwrap();
        let outcome = rep(page, async || {
            pointer::click(page, &target).await?;
            until(
                page,
                &format!(
                    "(() => {{ const t = document.querySelector({t}); return {selected}; }})()"
                ),
                "the pick",
            )
            .await
        })
        .await;
        record(rows, name(step), round, outcome);
    }
}

/// The rows of one control: the act and its undo, each its own row.
async fn exercise(page: &Page, path: &str, c: &Control, round: usize, rows: &mut Rows) {
    let sel = format!("[data-dp-{}]", c.id);
    let hit = format!("[data-dp-hit-{}]", c.id);
    let q = serde_json::to_string(&sel).unwrap();
    let el = format!("document.querySelector({q})");
    let name = |step: &str| format!("{path} | {} {step} | {} {}", c.kind, c.zone, c.name);
    // In view and quiet before the clock starts: the scroll is not the control's cost.
    let _ = page
        .evaluate(format!(
            "{el}?.scrollIntoView({{ block: 'center', behavior: 'instant' }})"
        ))
        .await;
    match c.kind.as_str() {
        "toggle" => {
            let state = format!(
                "(() => {{ const e = {el}; return String(e.checked ?? '') + e.getAttribute('aria-checked'); }})()"
            );
            for step in ["on", "off"] {
                let outcome = rep(page, async || {
                    let before: String = js(page, &state).await?;
                    pointer::click(page, &hit).await?;
                    until(
                        page,
                        &format!("{state} !== {}", serde_json::to_string(&before)?),
                        "the toggle",
                    )
                    .await
                })
                .await;
                record(rows, name(step), round, outcome);
            }
        }
        "type" => {
            let _ = page.evaluate(format!("{el}.focus()")).await;
            let value = format!("(() => {{ const e = {el}; return e.value ?? e.textContent; }})()");
            for step in ["key", "backspace"] {
                let outcome = rep(page, async || {
                    let before: String = js(page, &value).await?;
                    match step {
                        "key" => keyboard::type_text(page, "4").await?,
                        _ => keyboard::press(page, BACKSPACE).await?,
                    }
                    until(
                        page,
                        &format!("{value} !== {}", serde_json::to_string(&before)?),
                        "the keystroke",
                    )
                    .await
                })
                .await;
                record(rows, name(step), round, outcome);
            }
            let _ = keyboard::press(page, ESCAPE).await;
            let _ = page.evaluate("document.activeElement?.blur()").await;
        }
        "key" => {
            let _ = page.evaluate(format!("{el}.focus()")).await;
            let value = format!("{el}.getAttribute('aria-valuenow')");
            for (step, key, back) in [
                ("right", ARROW_RIGHT, ARROW_LEFT),
                ("left", ARROW_LEFT, ARROW_RIGHT),
            ] {
                let outcome = rep(page, async || {
                    let before: String = js(page, &format!("String({value})")).await?;
                    keyboard::press(page, key).await?;
                    // At an end the key does nothing: the other way round then.
                    if until(page, &format!("String({value}) !== {before:?}"), "the key")
                        .await
                        .is_err()
                    {
                        keyboard::press(page, back).await?;
                        until(page, &format!("String({value}) !== {before:?}"), "the key").await?;
                    }
                    Ok(())
                })
                .await;
                record(rows, name(step), round, outcome);
            }
            let _ = page.evaluate("document.activeElement?.blur()").await;
        }
        "drag" => {
            let value = format!("String({el}.getAttribute('aria-valuenow'))");
            // A separator's orientation is its line's, across the drag (todo 2162).
            let horizontal: bool = js(
                page,
                &format!(
                    "(() => {{ const e = {el}; const v = e.getAttribute('aria-orientation') === 'vertical'; \
                     return e.getAttribute('role') === 'separator' ? v : !v; }})()"
                ),
            )
            .await
            .unwrap_or(true);
            // At least one step of an index slider (0..3 over 460 px), towards the far end.
            let reach: f64 = js(
                page,
                &format!(
                    "(() => {{ const e = {el}; const n = (a) => Number(e.getAttribute(a)); \
                     const t = e.closest('[data-slot=track]')?.getBoundingClientRect(); \
                     const span = n('aria-valuemax') - n('aria-valuemin'); \
                     const px = t && span > 0 ? ({horizontal} ? t.width : t.height) / span : 0; \
                     const far = n('aria-valuenow') * 2 < n('aria-valuemax') + n('aria-valuemin') ? 1 : -1; \
                     return far * Math.max(60, Math.ceil(px)); }})()"
                ),
            )
            .await
            .unwrap_or(60.0);
            for (step, dx) in [("forth", reach), ("back", -reach)] {
                let outcome = rep(page, async || {
                    let before: String = js(page, &value).await?;
                    let from = pointer::centre_of(page, &sel).await?;
                    let to = match horizontal {
                        true => Point {
                            x: from.x + dx,
                            y: from.y,
                        },
                        false => Point {
                            x: from.x,
                            y: from.y - dx,
                        },
                    };
                    pointer::drag(page, from, to, 8).await?;
                    until(page, &format!("{value} !== {before:?}"), "the drag").await
                })
                .await;
                record(rows, name(step), round, outcome);
            }
        }
        "tab" => {
            let selected = "t.getAttribute('aria-selected') === 'true'";
            pick_and_back(
                page,
                &el,
                c,
                "[role=tablist]",
                "[role=tab]",
                selected,
                round,
                rows,
                name,
            )
            .await;
        }
        "segment" => {
            let selected = "(t.getAttribute('data-state') || '').split(' ').includes('checked')";
            pick_and_back(
                page,
                &el,
                c,
                "[data-slot=control]",
                "[data-slot=segment]",
                selected,
                round,
                rows,
                name,
            )
            .await;
        }
        "select" => {
            // A closed native select has no pointer path: the value set and its events sent.
            let next = |back: bool| {
                format!(
                    "(() => {{ const e = {el}; const n = e.options.length; \
                     e.selectedIndex = (e.selectedIndex + n + ({})) % n; \
                     e.dispatchEvent(new Event('input', {{ bubbles: true }})); \
                     e.dispatchEvent(new Event('change', {{ bubbles: true }})); }})()",
                    if back { -1 } else { 1 }
                )
            };
            for (step, back) in [("next", false), ("back", true)] {
                let outcome = rep(page, async || {
                    page.evaluate(next(back)).await?;
                    Ok(())
                })
                .await;
                record(rows, name(step), round, outcome);
            }
        }
        "popup" => {
            let open = format!(
                "({el}.getAttribute('aria-expanded') === 'true' || !!document.querySelector('[role=dialog], [role=menu], [role=listbox]:not(#docs-main [role=listbox])'))"
            );
            let outcome = rep(page, async || {
                pointer::click(page, &hit).await?;
                until(page, &open, "the popup to open").await
            })
            .await;
            let opened = outcome.is_ok();
            record(rows, name("open"), round, outcome);
            if opened {
                let outcome = rep(page, async || {
                    keyboard::press(page, ESCAPE).await?;
                    until(page, &format!("!{open}"), "the popup to close").await
                })
                .await;
                record(rows, name("close"), round, outcome);
            }
        }
        _ => {
            let outcome = rep(page, async || pointer::click(page, &hit).await).await;
            record(rows, name("click"), round, outcome);
        }
    }
    // Whatever a control opened or where it went, back to the page as it was.
    let _ = keyboard::press(page, ESCAPE).await;
    let _ = page.evaluate("document.activeElement?.blur()").await;
    let elsewhere = js::<bool>(
        page,
        &format!(
            "location.pathname !== {}",
            serde_json::to_string(path).unwrap()
        ),
    )
    .await
    .unwrap_or(false);
    if elsewhere {
        let _ = ensure(page, path).await;
    }
}

async fn theme(page: &Page, path: &str, round: usize, rows: &mut Rows) {
    for step in ["dark", "light"] {
        let outcome = rep(page, async || {
            let before: String = js(page, SCHEME).await?;
            pointer::click(page, THEME).await?;
            until(
                page,
                &format!("{SCHEME} !== {}", serde_json::to_string(&before)?),
                "the scheme",
            )
            .await
        })
        .await;
        record(
            rows,
            format!("{path} | theme {step} | header"),
            round,
            outcome,
        );
    }
}

/// One wheel notch down, one back up.
async fn scroll(page: &Page, path: &str, round: usize, rows: &mut Rows) {
    let at = Point { x: 900.0, y: 450.0 };
    for (step, dy) in [("down", 240.0), ("up", -240.0)] {
        let outcome = rep(page, async || {
            let before: f64 = js(page, &format!("{SCROLLER}.scrollTop")).await?;
            page.execute(
                DispatchMouseEventParams::builder()
                    .r#type(DispatchMouseEventType::MouseWheel)
                    .x(at.x)
                    .y(at.y)
                    .delta_x(0.0)
                    .delta_y(dy)
                    .build()
                    .map_err(anyhow::Error::msg)?,
            )
            .await?;
            until(
                page,
                &format!("{SCROLLER}.scrollTop !== {before}"),
                "the wheel",
            )
            .await
        })
        .await;
        record(
            rows,
            format!("{path} | scroll {step} | page"),
            round,
            outcome,
        );
    }
}

#[derive(Serialize)]
struct Summary {
    row: String,
    n: usize,
    first_task: Option<f64>,
    task: f64,
    task_min: f64,
    /// The mean of the fastest third: load only adds time, and one lucky rep is no figure.
    task_low: f64,
    task_max: f64,
    script: f64,
    layout: f64,
    frame: f64,
    long: u32,
    writes: u32,
    nodes: u32,
    invalid: u32,
    failed: usize,
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn fastest_third(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len().div_ceil(3);
    v[..n].iter().sum::<f64>() / n as f64
}

fn summarise(name: &str, row: &Row) -> Summary {
    // A single round leaves only the first visit to report.
    let reps = if row.warm.is_empty() {
        &row.first
    } else {
        &row.warm
    };
    let pick = |f: fn(&Rep) -> f64| reps.iter().map(f).collect::<Vec<_>>();
    Summary {
        row: name.to_string(),
        n: reps.len(),
        first_task: row.first.first().map(|r| r.task),
        task: median(pick(|r| r.task)),
        task_min: pick(|r| r.task).into_iter().fold(f64::INFINITY, f64::min),
        task_low: fastest_third(pick(|r| r.task)),
        task_max: pick(|r| r.task).into_iter().fold(0.0, f64::max),
        script: median(pick(|r| r.script)),
        layout: median(pick(|r| r.layout)),
        frame: median(pick(|r| r.frame)),
        long: reps.iter().map(|r| r.long).max().unwrap_or(0),
        writes: reps.iter().map(|r| r.writes).max().unwrap_or(0),
        nodes: reps.iter().map(|r| r.nodes).max().unwrap_or(0),
        invalid: row.invalid,
        failed: row.failed.len(),
    }
}

fn out_path() -> PathBuf {
    std::env::var_os("DOCS_PERF_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let exe = std::env::current_exe().unwrap();
            // <target>/<profile>/deps/<binary>
            exe.ancestors().nth(3).unwrap().join("docs-perf.json")
        })
}

fn summaries(rows: &Rows) -> Vec<Summary> {
    let mut all: Vec<Summary> = rows.iter().map(|(k, v)| summarise(k, v)).collect();
    all.sort_by(|a, b| b.task.total_cmp(&a.task));
    all
}

/// Writes the JSON and markdown reports; `print` also prints every row. With two variants
/// it adds B against A per row: the min is the steadier figure on a loaded machine.
fn report(variants: &[Variant], rounds: &[RoundMeta], print: bool) -> Result<()> {
    let path = out_path();
    let mut json = serde_json::Map::new();
    let mut md = String::new();
    for v in variants {
        md.push_str(&format!("- {}: {} build `{}`\n", v.label, v.base, v.build));
    }
    let loads: Vec<f64> = rounds
        .iter()
        .flat_map(|r| [r.load_before, r.load_after])
        .collect();
    md.push_str(&format!(
        "- load {:.1}..{:.1} over {} variant rounds\n\n",
        loads.iter().copied().fold(f64::INFINITY, f64::min),
        loads.iter().copied().fold(0.0, f64::max),
        rounds.len()
    ));
    for v in variants {
        let all = summaries(&v.rows);
        print_rows(v, &all, print && variants.len() == 1);
        if variants.len() > 1 {
            md.push_str(&format!("### {}\n\n", v.label));
        }
        md.push_str(&table(&all));
        md.push('\n');
        json.insert(
            v.label.to_string(),
            serde_json::json!({ "base": v.base, "build": v.build, "summary": all, "raw": v.rows }),
        );
    }
    if let [a, b] = variants {
        md.push_str(&compare(a, b, print));
    }
    json.insert("rounds".into(), serde_json::to_value(rounds)?);
    std::fs::write(&path, serde_json::to_string_pretty(&json)?)
        .with_context(|| format!("write {}", path.display()))?;
    std::fs::write(path.with_extension("md"), md)?;
    eprintln!("docs-perf: report written to {}", path.display());
    Ok(())
}

/// B against A for every row both measured, largest change of the fastest third first: on
/// one build served twice at load 6-12 that moved 0-2 ms a row, the median up to 9 (todo 2236).
fn compare(a: &Variant, b: &Variant, print: bool) -> String {
    let theirs: BTreeMap<String, Summary> = summaries(&b.rows)
        .into_iter()
        .map(|s| (s.row.clone(), s))
        .collect();
    let mut pairs: Vec<(Summary, &Summary)> = summaries(&a.rows)
        .into_iter()
        .filter_map(|s| theirs.get(&s.row).map(|t| (s, t)))
        .collect();
    let change = |(x, y): &(Summary, &Summary)| (y.task_low - x.task_low).abs();
    pairs.sort_by(|p, q| change(q).total_cmp(&change(p)));
    let mut md = String::from(
        "### B - A\n\n| page | kind | control | n | A low | B low | delta low | A task | B task | delta | A min | B min | A layout | B layout |\n\
         |---|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|\n",
    );
    for (x, y) in &pairs {
        if print {
            println!(
                "docs-perf B-A | {:<70} | low {:>7.1} -> {:>7.1} ({:>+6.1}) | task {:>7.1} -> {:>7.1} ({:>+6.1}) | min {:>7.1} -> {:>7.1} | layout {:>6.1} -> {:>6.1}",
                x.row,
                x.task_low,
                y.task_low,
                y.task_low - x.task_low,
                x.task,
                y.task,
                y.task - x.task,
                x.task_min,
                y.task_min,
                x.layout,
                y.layout,
            );
        }
        md.push_str(&format!(
            "| {} | {}/{} | {:.1} | {:.1} | {:+.1} | {:.1} | {:.1} | {:+.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            x.row.split(" | ").collect::<Vec<_>>().join(" | "),
            x.n,
            y.n,
            x.task_low,
            y.task_low,
            y.task_low - x.task_low,
            x.task,
            y.task,
            y.task - x.task,
            x.task_min,
            y.task_min,
            x.layout,
            y.layout,
        ));
    }
    md
}

fn print_rows(v: &Variant, all: &[Summary], print: bool) {
    for s in all.iter().filter(|_| print) {
        println!(
            "docs-perf | {:<70} | n {:>2} | first {:>7.1} | task {:>7.1} [{:>6.1}..{:>6.1}] low {:>6.1} | script {:>6.1} | layout {:>6.1} | frame {:>5.1} | long {:>2} | writes {:>5} | nodes {:>5} | invalid {} | failed {}",
            s.row,
            s.n,
            s.first_task.unwrap_or(f64::NAN),
            s.task,
            s.task_min,
            s.task_max,
            s.task_low,
            s.script,
            s.layout,
            s.frame,
            s.long,
            s.writes,
            s.nodes,
            s.invalid,
            s.failed,
        );
    }
    for (name, row) in v.rows.iter().filter(|_| print) {
        if let Some(e) = row.failed.first() {
            println!("docs-perf failed | {name} | {e}");
        }
    }
}

/// The rows as a markdown table, for a brain note.
fn table(all: &[Summary]) -> String {
    let mut md = String::from(
        "| page | kind | control | n | first | task | min | low | max | script | layout | frame | long | writes | nodes | invalid | failed |\n\
         |---|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|\n",
    );
    for s in all {
        let cells: Vec<&str> = s.row.split(" | ").collect();
        md.push_str(&format!(
            "| {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {} | {} | {} | {} | {} |\n",
            cells.join(" | "),
            s.n,
            s.first_task.unwrap_or(f64::NAN),
            s.task,
            s.task_min,
            s.task_low,
            s.task_max,
            s.script,
            s.layout,
            s.frame,
            s.long,
            s.writes,
            s.nodes,
            s.invalid,
            s.failed,
        ));
    }
    md
}

/// The served wasm's file name, the build's identity in the report.
const WASM_NAME: &str = "performance.getEntriesByType('resource').map((e) => e.name) \
    .filter((n) => n.endsWith('.wasm')).map((n) => n.split('/').pop()).join(' ')";

/// One pass over every page: its mount, plain twin, theme, scroll and controls.
async fn survey_round(
    page: &Page,
    pages: &[String],
    kinds: &Option<Vec<String>>,
    per_kind: usize,
    round: usize,
    rows: &mut Rows,
) {
    let started = Instant::now();
    for (n, path) in pages.iter().enumerate() {
        eprintln!(
            "docs-perf: round {round} page {n}/{} {path} at {:.0} s",
            pages.len(),
            started.elapsed().as_secs_f64()
        );
        if wanted(kinds, "mount") || wanted(kinds, "plain") {
            // The nav scrolled to the link untimed: the row prices the click.
            let _ = aim(page, path).await;
            let outcome = rep(page, async || navigate(page, path).await).await;
            record(rows, format!("{path} | mount | page"), round, outcome);
            if let Err(e) = ensure(page, path).await {
                eprintln!("docs-perf: {path} skipped: {e:#}");
                continue;
            }
        } else if let Err(e) = ensure(page, path).await {
            eprintln!("docs-perf: {path} skipped: {e:#}");
            continue;
        }
        if wanted(kinds, "plain") {
            let outcome = rep(page, async || plain(page).await).await;
            record(rows, format!("{path} | plain | page"), round, outcome);
            let _ = unplain(page).await;
        }
        if wanted(kinds, "theme") {
            theme(page, path, round, rows).await;
        }
        if wanted(kinds, "scroll") {
            scroll(page, path, round, rows).await;
        }
        // Tagged again before each: a tab switch remounts its panel and drops the tags.
        let found = controls(page, per_kind).await.unwrap_or_default().len();
        for i in 0..found {
            let now = controls(page, per_kind).await.unwrap_or_default();
            if let Some(c) = now.get(i).filter(|c| wanted(kinds, &c.kind)) {
                exercise(page, path, c, round, rows).await;
                let after = format!("{path} {} {}", c.kind, c.name);
                if let Err(e) = revive(page, path, &after).await {
                    eprintln!("docs-perf: {path} left: {e:#}");
                    break;
                }
            }
        }
    }
}

/// Every nav page mounted in-app, its plain-DOM twin, the header theme switch, a page
/// scroll and the controls on it, `DOCS_PERF_ROUNDS` times over.
#[test]
#[ignore = "docs timing survey, run on request"]
fn survey() {
    block_on(async {
        let only = env_list("DOCS_PERF_PAGES");
        let kinds = env_list("DOCS_PERF_KINDS");
        let rounds = env_usize("DOCS_PERF_ROUNDS", 4);
        let per_kind = env_usize("DOCS_PERF_PER_KIND", 2);
        let pages: Vec<String> = e2e::sweep::discover()
            .await
            .unwrap()
            .into_iter()
            .filter(|p| p != "/" && p != "/no-such-page")
            .filter(|p| {
                only.as_ref()
                    .is_none_or(|o| o.iter().any(|o| p.contains(o.as_str())))
            })
            .collect();
        eprintln!("docs-perf: {} pages, {rounds} rounds", pages.len());
        let start = pages.first().cloned().expect("no page picked");
        let fixture = Fixture::open_until(&start, Viewport::Desktop, Scheme::Light, READY)
            .await
            .unwrap();
        let page = &fixture.page;
        let front = frames::bring_to_front(page).await.unwrap();
        page.execute(EnableParams::default()).await.unwrap();
        let mut variants = vec![Variant {
            label: "A",
            base: e2e::base_url(),
            build: String::new(),
            rows: Rows::new(),
        }];
        // B's own tab: a reload per round would make its warm rows pay first insertion (todo 2288).
        let mut other = None;
        if let Ok(b) = std::env::var("DOCS_PERF_B") {
            variants.push(Variant {
                label: "B",
                base: b.trim_end_matches('/').to_string(),
                build: String::new(),
                rows: Rows::new(),
            });
            let tab = Fixture::open_until(&start, Viewport::Desktop, Scheme::Light, READY)
                .await
                .unwrap();
            tab.page.execute(EnableParams::default()).await.unwrap();
            other = Some(tab);
        }
        let tabs: Vec<&Page> = std::iter::once(page)
            .chain(other.as_ref().map(|tab| &tab.page))
            .collect();
        // The page before the first: a mount of `start` from itself would be no navigation.
        let lead = pages.get(1).cloned().unwrap_or_else(|| "/".to_string());
        let mut metas = Vec::new();
        for round in 0..rounds {
            // A B, then B A: a load drift over the run hits both alike.
            let order: Vec<usize> = (0..variants.len()).collect();
            let order = if round % 2 == 1 {
                order.into_iter().rev().collect()
            } else {
                order
            };
            for i in order {
                let page = tabs[i];
                let v = &mut variants[i];
                *BASE.lock().unwrap() = v.base.clone();
                page.bring_to_front().await.unwrap();
                let started = Instant::now();
                let load_before = load_average();
                // Both tabs opened on A's server: B loads its own on the first round.
                if round == 0 {
                    if i == 0 {
                        page.evaluate(RECORDER).await.unwrap();
                        expand_nav(page).await.unwrap();
                    } else {
                        load(page, &start).await.unwrap();
                    }
                    let _ = navigate(page, &lead).await;
                }
                if v.build.is_empty() {
                    v.build = js(page, WASM_NAME).await.unwrap_or_default();
                }
                survey_round(page, &pages, &kinds, per_kind, round, &mut v.rows).await;
                let secs = started.elapsed().as_secs_f64();
                eprintln!("docs-perf: round {round} {} in {secs:.0} s", v.label);
                metas.push(RoundMeta {
                    round,
                    variant: v.label,
                    load_before,
                    load_after: load_average(),
                    secs,
                });
            }
            // After every round, so a cut-short run still leaves its numbers.
            report(&variants, &metas, round + 1 == rounds).unwrap();
        }
        front.release().await.unwrap();
        if let Some(tab) = other {
            let _ = tab
                .close_allowing("a survey clicks whatever it finds")
                .await;
        }
        let _ = fixture
            .close_allowing("a survey clicks whatever it finds")
            .await;
    });
}
