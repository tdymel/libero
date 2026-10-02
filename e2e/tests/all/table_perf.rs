//! Main-thread time of large `Table`s in a real browser (todo 1462): mounting 100, 1000 and
//! 5000 rows, then sorting, filtering, selecting, expanding, resizing, reordering, paging and
//! scrolling. Each row is repeated and printed as `timing | ...`, median / worst in ms; a
//! budget far above the measured median trips only a real regression. Opt-in, a shared
//! CPU makes milliseconds noise in the gate; run it on the release build:
//! `E2E_RELEASE=1 cargo run -p e2e -- table_perf:: --ignored --nocapture`.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType,
};
use chromiumoxide::cdp::browser_protocol::performance::{EnableParams, GetMetricsParams};
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, BACKSPACE};
use e2e::passes::pointer::{self, Point};
use e2e::{Fixture, Viewport, frames, wait};
use serde::Deserialize;

/// Unmeasured passes first: the first run of a path pays for its wasm and style warm-up.
const WARM_UP: usize = 2;

/// Reps per row.
const REPS: usize = 12;

/// Median task budgets by row name prefix, ms: about 5x the medians of 2026-10-05.
const BUDGETS: &[(&str, f64)] = &[
    ("mount", 5_000.0),
    ("unmount", 1_000.0),
    ("sort", 2_000.0),
    ("select", 2_500.0),
    ("detail", 500.0),
    ("resize", 3_000.0),
    ("column", 1_000.0),
    ("filter", 500.0),
    ("next", 500.0),
    ("wheel", 100.0),
    ("jump", 100.0),
];

/// Frame gaps, long animation frames and slow events while a rep runs.
const RECORDER: &str = r#"(() => {
    const t = { frames: [], loaf: [], events: [], last: null };
    const tick = (now) => {
        if (t.last !== null) t.frames.push(now - t.last);
        t.last = now;
        requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    try {
        new PerformanceObserver((l) => t.loaf.push(...l.getEntries().map((e) => e.duration)))
            .observe({ type: 'long-animation-frame' });
    } catch (_) {}
    try {
        new PerformanceObserver((l) => t.events.push(...l.getEntries().map((e) => e.duration)))
            .observe({ type: 'event', durationThreshold: 16 });
    } catch (_) {}
    t.reset = () => { t.frames = []; t.loaf = []; t.events = []; };
    t.read = () => JSON.stringify({
        frame: Math.max(0, ...t.frames),
        loaf: t.loaf.length,
        event: Math.max(0, ...t.events),
    });
    window.__timing = t;
})()"#;

/// One interaction, milliseconds.
#[derive(Debug, Clone, Copy, Default)]
struct Rep {
    /// Every main-thread task: script, style, layout, paint.
    task: f64,
    script: f64,
    /// Style recalculation and layout.
    layout: f64,
    /// The longest frame while it ran: over 16.7 is a dropped frame.
    frame: f64,
    /// Long animation frames (50 ms and over).
    long: usize,
    /// The slowest event's input-to-paint time; 0 under the API's 16 ms floor.
    event: f64,
}

#[derive(Deserialize)]
struct Seen {
    frame: f64,
    loaf: usize,
    event: f64,
}

/// `name -> seconds` for the counters a rep reads.
async fn metrics(page: &Page) -> Result<BTreeMap<String, f64>> {
    let reply = page.execute(GetMetricsParams::default()).await?;
    Ok(reply
        .result
        .metrics
        .iter()
        .map(|metric| (metric.name.clone(), metric.value))
        .collect())
}

fn delta_ms(after: &BTreeMap<String, f64>, before: &BTreeMap<String, f64>, key: &str) -> f64 {
    (after.get(key).copied().unwrap_or(0.0) - before.get(key).copied().unwrap_or(0.0)) * 1e3
}

/// Until 60 ms pass with under 2 ms of main-thread work: effects and the passes an
/// interaction starts belong to it.
async fn quiet(page: &Page) -> Result<()> {
    let started = Instant::now();
    let mut last = metrics(page).await?;
    loop {
        tokio::time::sleep(Duration::from_millis(60)).await;
        let now = metrics(page).await?;
        if delta_ms(&now, &last, "TaskDuration") < 2.0 {
            return Ok(());
        }
        if started.elapsed() > Duration::from_secs(20) {
            bail!("the page never went quiet");
        }
        last = now;
    }
}

/// Reps by row name.
type Rows = BTreeMap<String, Vec<Rep>>;

/// Runs `act` `WARM_UP + reps` times; each run names the row it belongs to.
async fn measure(
    page: &Page,
    reps: usize,
    mut act: impl AsyncFnMut(usize) -> Result<String>,
) -> Result<Rows> {
    let mut rows = Rows::new();
    for i in 0..WARM_UP + reps {
        quiet(page).await?;
        page.evaluate("window.__timing.reset()").await?;
        let before = metrics(page).await?;
        let row = act(i).await?;
        quiet(page).await?;
        let after = metrics(page).await?;
        let seen: Seen =
            serde_json::from_str(&js::<String>(page, "window.__timing.read()").await?)?;
        if i < WARM_UP {
            continue;
        }
        rows.entry(row).or_default().push(Rep {
            task: delta_ms(&after, &before, "TaskDuration"),
            script: delta_ms(&after, &before, "ScriptDuration"),
            layout: delta_ms(&after, &before, "LayoutDuration")
                + delta_ms(&after, &before, "RecalcStyleDuration"),
            frame: seen.frame,
            long: seen.loaf,
            event: seen.event,
        });
    }
    Ok(rows)
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Prints each row as median / worst, then holds its median task time to its budget.
fn report(table: &str, rows: &Rows, budgets: &[(&str, f64)]) {
    let mut over = Vec::new();
    for (row, reps) in rows {
        let pick = |f: fn(&Rep) -> f64| reps.iter().map(f).collect::<Vec<_>>();
        let worst = |f: fn(&Rep) -> f64| pick(f).into_iter().fold(0.0, f64::max);
        let task = median(pick(|r| r.task));
        println!(
            "timing | {:<34} | n {:>2} | task {:>7.1} / {:>7.1} | script {:>7.1} / {:>7.1} | \
             layout {:>6.1} / {:>6.1} | frame {:>6.1} / {:>6.1} | long {:>2} | event {:>6.1} / {:>6.1}",
            format!("{table} {row}"),
            reps.len(),
            task,
            worst(|r| r.task),
            median(pick(|r| r.script)),
            worst(|r| r.script),
            median(pick(|r| r.layout)),
            worst(|r| r.layout),
            median(pick(|r| r.frame)),
            worst(|r| r.frame),
            reps.iter().map(|r| r.long).sum::<usize>(),
            median(pick(|r| r.event)),
            worst(|r| r.event),
        );
        let budget = budgets
            .iter()
            .find(|(listed, _)| row.starts_with(listed))
            .map(|(_, budget)| *budget)
            .unwrap_or_else(|| panic!("{table} {row}: no budget"));
        if task > budget {
            over.push(format!(
                "{table} {row}: median task {task:.1} ms (budget {budget} ms)"
            ));
        }
    }
    assert!(over.is_empty(), "over budget: {over:?}");
}

/// The page in front, the recorder and the CDP counters on; `run` gets the page.
async fn timed(route: &str, run: impl AsyncFnOnce(&Page) -> Result<()>) {
    let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
    let page = &fixture.page;
    let front = frames::bring_to_front(page).await.unwrap();
    let outcome = async {
        page.execute(EnableParams::default()).await?;
        page.evaluate(RECORDER).await?;
        run(page).await
    }
    .await;
    front.release().await.unwrap();
    outcome.unwrap();
    fixture.console.assert_clean(route).unwrap();
    fixture.close().await.unwrap();
}

async fn js<T: serde::de::DeserializeOwned>(page: &Page, expression: &str) -> Result<T> {
    Ok(page.evaluate(expression).await?.into_value()?)
}

/// Clicks `selector`, then waits until `changed` (a JS string expression) differs.
async fn click_until_changed(page: &Page, selector: &str, changed: &str) -> Result<()> {
    let before: String = js(page, changed).await?;
    pointer::click(page, selector).await?;
    wait::for_js_true(
        page,
        &format!("{changed} !== {}", serde_json::to_string(&before)?),
        &format!("a click on {selector} to change {changed}"),
    )
    .await
}

/// Drags `selector` by `by` px sideways in `moves` steps, back again on odd reps, until
/// `changed` differs.
async fn drag_until_changed(
    page: &Page,
    i: usize,
    selector: &str,
    by: f64,
    changed: &str,
) -> Result<()> {
    let sign = if i.is_multiple_of(2) { 1.0 } else { -1.0 };
    let from = pointer::centre_of(page, selector).await?;
    let to = Point {
        x: from.x + by * sign,
        y: from.y,
    };
    let before: String = js(page, changed).await?;
    pointer::drag(page, from, to, 10).await?;
    wait::for_js_true(
        page,
        &format!("{changed} !== {}", serde_json::to_string(&before)?),
        &format!("a drag of {selector} to change {changed}"),
    )
    .await
}

/// One key per rep into the quick filter: `text` typed, then deleted again.
async fn filter_keys(page: &Page, text: &str) -> Result<Rows> {
    const FIELD: &str = "input[type=search]";
    pointer::click(page, FIELD).await?;
    let chars: Vec<char> = text.chars().collect();
    let cycle = 2 * chars.len();
    measure(page, REPS, async |i| {
        let step = i % cycle;
        let expected: String = match step < chars.len() {
            true => {
                keyboard::type_text(page, &chars[step].to_string()).await?;
                chars[..=step].iter().collect()
            }
            false => {
                keyboard::press(page, BACKSPACE).await?;
                chars[..cycle - step - 1].iter().collect()
            }
        };
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{FIELD}').value === {}",
                serde_json::to_string(&expected)?
            ),
            "the keystroke",
        )
        .await?;
        Ok("filter keystroke".to_string())
    })
    .await
}

/// The header cell of column `name`, by its text: `aria-label` is set only with column menus.
fn header(name: &str) -> String {
    format!(
        "[...document.querySelectorAll('thead th')].find((th) => th.textContent.trim().startsWith('{name}'))"
    )
}

/// Tags the element `expression` finds with `data-e2e="{tag}"`, for a selector.
async fn tag(page: &Page, expression: &str, tag: &str) -> Result<String> {
    page.evaluate(format!("{expression}.setAttribute('data-e2e', '{tag}')"))
        .await?;
    Ok(format!("[data-e2e={tag}]"))
}

/// Header clicks cycle a column ascending, descending, then unsorted.
async fn sort_reps(page: &Page, column: &str) -> Result<Rows> {
    let th = header(column);
    let button = tag(
        page,
        &format!("{th}.querySelector('[data-sort-button]')"),
        "sort",
    )
    .await?;
    measure(page, REPS, async |_| {
        click_until_changed(
            page,
            &button,
            &format!("String({th}.getAttribute('aria-sort'))"),
        )
        .await?;
        Ok("sort click".to_string())
    })
    .await
}

const FIRST_ROW_SELECTED: &str =
    "String(document.querySelector('tbody tr').getAttribute('aria-selected'))";

/// A row's box, then select-all, each toggled on and off.
async fn select_reps(page: &Page) -> Result<Rows> {
    let mut rows = measure(page, REPS, async |_| {
        click_until_changed(
            page,
            "tbody tr:nth-child(3) [data-select] input",
            "String(document.querySelector('tbody tr:nth-child(3)').getAttribute('aria-selected'))",
        )
        .await?;
        Ok("select row".to_string())
    })
    .await?;
    rows.extend(
        measure(page, REPS, async |_| {
            click_until_changed(page, "thead [data-select] input", FIRST_ROW_SELECTED).await?;
            Ok("select all".to_string())
        })
        .await?,
    );
    Ok(rows)
}

#[test]
#[ignore = "table timing report, run on request"]
fn mounting() {
    block_on(timed("/table-perf/mount", async |page| {
        for kind in ["plain", "full"] {
            for n in [100, 1000, 5000] {
                let rows = measure(page, 2 * REPS, async |i| {
                    let (button, shown, row) = match i % 2 {
                        0 => (format!("#{kind}-{n}"), format!("{kind}-{n}"), "mount"),
                        _ => ("#unmount".to_string(), "none".to_string(), "unmount"),
                    };
                    pointer::click(page, &button).await?;
                    wait::for_js_true(
                        page,
                        &format!(
                            "document.getElementById('shown').dataset.shown === '{shown}' \
                             && document.querySelectorAll('tbody tr').length === {}",
                            if shown == "none" { 0 } else { n }
                        ),
                        &format!("{row} of {kind} {n}"),
                    )
                    .await?;
                    Ok(row.to_string())
                })
                .await?;
                report(&format!("Table {kind} {n}"), &rows, BUDGETS);
            }
        }
        Ok(())
    }));
}

/// 1000 rows with selection, details, column menus, resizable columns and a quick filter.
#[test]
#[ignore = "table timing report, run on request"]
fn interacting_with_a_full_table() {
    block_on(timed("/table-perf/full", async |page| {
        let mut rows = sort_reps(page, "Age").await?;
        rows.extend(select_reps(page).await?);
        rows.extend(
            measure(page, REPS, async |_| {
                click_until_changed(
                    page,
                    "tbody tr:nth-child(2) [data-detail-button]",
                    "document.querySelector('tbody tr:nth-child(2) [data-detail-button]').getAttribute('aria-expanded')",
                )
                .await?;
                Ok("detail toggle".to_string())
            })
            .await?,
        );
        let city = header("City");
        let handle = tag(
            page,
            &format!("{city}.querySelector('[data-resize-handle]')"),
            "resize",
        )
        .await?;
        rows.extend(
            measure(page, REPS, async |i| {
                drag_until_changed(page, i, &handle, 60.0, &format!("{city}.style.width")).await?;
                Ok("resize drag".to_string())
            })
            .await?,
        );
        report("Table full 1000", &rows, BUDGETS);
        let order = "[...document.querySelectorAll('thead th')].map((th) => th.textContent.trim()).join('|')";
        let grip = tag(
            page,
            &format!("{}.querySelector('[data-drag-handle]')", header("Role")),
            "grip",
        )
        .await?;
        // Role over City's far edge and back over its near one.
        let target = format!(
            "(() => {{ const city = {}.getBoundingClientRect(); const grip = document.querySelector('{grip}').getBoundingClientRect(); \
             return grip.x < city.x ? city.right - 4 - (grip.x + grip.width / 2) : city.x + 4 - (grip.x + grip.width / 2); }})()",
            header("City")
        );
        let rows = measure(page, REPS, async |_| {
            let by: f64 = js(page, &target).await?;
            drag_until_changed(page, 0, &grip, by, order).await?;
            Ok("column drag".to_string())
        })
        .await?;
        report("Table full 1000", &rows, BUDGETS);
        report("Table full 1000", &filter_keys(page, "ali").await?, BUDGETS);
        Ok(())
    }));
}

/// 5000 rows, 50 a page: next page, and a sort that goes back to page 1.
#[test]
#[ignore = "table timing report, run on request"]
fn paging() {
    block_on(timed("/table-perf/paged", async |page| {
        let range = "document.querySelector('[data-slot=range]').textContent";
        let next = tag(
            page,
            "[...document.querySelectorAll('button[aria-label]')].find((b) => /next/i.test(b.getAttribute('aria-label')))",
            "next",
        )
        .await?;
        // Below the rows: in view first, where every page keeps it.
        page.evaluate(format!(
            "document.querySelector('{next}').scrollIntoView({{ block: 'center' }})"
        ))
        .await?;
        let rows = measure(page, REPS, async |_| {
            click_until_changed(page, &next, range).await?;
            Ok("next page".to_string())
        })
        .await?;
        report("Table paged 5000", &rows, BUDGETS);
        page.evaluate("window.scrollTo(0, 0)").await?;
        let mut rows = sort_reps(page, "Age").await?;
        rows.extend(select_reps(page).await?);
        report("Table paged 5000", &rows, BUDGETS);
        Ok(())
    }));
}

/// 10k windowed rows: wheel notches and long jumps, a sort, select-all and filtering.
#[test]
#[ignore = "table timing report, run on request"]
fn a_windowed_table() {
    const SCROLLER: &str = "[...document.querySelectorAll('[data-table-scroll] *, [data-table-scroll]')].find((el) => el.scrollHeight > el.clientHeight + 1)";
    block_on(timed("/table-perf/windowed", async |page| {
        let at: Point = js(
            page,
            &format!("(() => {{ const r = {SCROLLER}.getBoundingClientRect(); return {{ x: r.x + r.width / 2, y: r.y + r.height / 2 }}; }})()"),
        )
        .await?;
        let mut rows = measure(page, REPS, async |_| {
            let before: f64 = js(page, &format!("{SCROLLER}.scrollTop")).await?;
            page.execute(
                DispatchMouseEventParams::builder()
                    .r#type(DispatchMouseEventType::MouseWheel)
                    .x(at.x)
                    .y(at.y)
                    .delta_x(0.0)
                    .delta_y(120.0)
                    .build()
                    .map_err(anyhow::Error::msg)?,
            )
            .await?;
            wait::for_js_true(
                page,
                &format!("{SCROLLER}.scrollTop !== {before}"),
                "the wheel to scroll",
            )
            .await?;
            Ok("wheel".to_string())
        })
        .await?;
        rows.extend(
            measure(page, REPS, async |_| {
                // A scrollbar drag's worth: rows none of which were drawn.
                let top: f64 = js(page, &format!("{SCROLLER}.scrollTop += 4000")).await?;
                let first = (top / 40.0) as usize;
                wait::for_js_true(
                    page,
                    &format!(
                        "[...document.querySelectorAll('tbody th')].some((th) => th.textContent.endsWith(' {}'))",
                        first + 2
                    ),
                    "the rows in view",
                )
                .await?;
                Ok("jump".to_string())
            })
            .await?,
        );
        page.evaluate(format!("{SCROLLER}.scrollTop = 0")).await?;
        rows.extend(sort_reps(page, "Age").await?);
        rows.extend(
            measure(page, REPS, async |_| {
                click_until_changed(page, "thead [data-select] input", FIRST_ROW_SELECTED).await?;
                Ok("select all".to_string())
            })
            .await?,
        );
        rows.extend(filter_keys(page, "ali").await?);
        report("Table windowed 10k", &rows, BUDGETS);
        Ok(())
    }));
}
