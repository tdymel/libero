//! Main-thread time of large `Table`s in a real browser (todo 1462): mounting 100, 1000 and
//! 5000 rows, then sorting, filtering, selecting, expanding, resizing, reordering, paging and
//! scrolling. Each row is repeated and printed as `timing | ...`, median / worst in ms; a
//! budget far above the measured median trips only a real regression. Opt-in, a shared
//! CPU makes milliseconds noise in the gate; run it on the release build:
//! `E2E_RELEASE=1 cargo run -p e2e -- table_perf:: --ignored --nocapture`.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, BACKSPACE};
use e2e::passes::pointer::{self, Point};
use e2e::wait;

use super::perf::timing::{Rows, SCROLLER, drag_reps, js, measure, report, timed, wheel_reps};

/// Reps per row.
const REPS: usize = 12;

/// Median task budgets by row, ms: about 5x the medians of 2026-10-05.
const BUDGETS: &[(&str, f64)] = &[
    ("mount", 5_000.0),
    ("unmount", 1_000.0),
    ("sort click", 2_000.0),
    ("select row", 2_500.0),
    ("select all", 2_500.0),
    ("detail toggle", 500.0),
    // A column resize.
    ("drag", 3_000.0),
    ("column drag", 1_000.0),
    ("filter keystroke", 500.0),
    ("next page", 500.0),
    ("wheel", 100.0),
    ("jump", 100.0),
];

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
        Ok("filter keystroke")
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
        Ok("sort click")
    })
    .await
}

const FIRST_ROW_SELECTED: &str =
    "String(document.querySelector('tbody tr').getAttribute('aria-selected'))";

async fn select_all_reps(page: &Page) -> Result<Rows> {
    measure(page, REPS, async |_| {
        click_until_changed(
            page,
            "thead [data-select] span[aria-hidden]",
            FIRST_ROW_SELECTED,
        )
        .await?;
        Ok("select all")
    })
    .await
}

/// A row's box, then select-all, each toggled on and off.
async fn select_reps(page: &Page, table: &str) -> Result<()> {
    let rows = measure(page, REPS, async |_| {
        click_until_changed(
            page,
            "tbody tr:nth-child(3) [data-select] span[aria-hidden]",
            "String(document.querySelector('tbody tr:nth-child(3)').getAttribute('aria-selected'))",
        )
        .await?;
        Ok("select row")
    })
    .await?;
    report(table, &rows, BUDGETS);
    report(table, &select_all_reps(page).await?, BUDGETS);
    Ok(())
}

/// Role's grip dragged over City's far edge, and back over its near one.
async fn column_drag_reps(page: &Page) -> Result<Rows> {
    let order =
        "[...document.querySelectorAll('thead th')].map((th) => th.textContent.trim()).join('|')";
    let grip = tag(
        page,
        &format!("{}.querySelector('[data-drag-handle]')", header("Role")),
        "grip",
    )
    .await?;
    let target = format!(
        "(() => {{ const city = {}.getBoundingClientRect(); const grip = document.querySelector('{grip}').getBoundingClientRect(); \
         return grip.x < city.x ? city.right - 4 - (grip.x + grip.width / 2) : city.x + 4 - (grip.x + grip.width / 2); }})()",
        header("City")
    );
    measure(page, REPS, async |_| {
        let by: f64 = js(page, &target).await?;
        let from = pointer::centre_of(page, &grip).await?;
        let before: String = js(page, order).await?;
        let to = Point {
            x: from.x + by,
            y: from.y,
        };
        pointer::drag(page, from, to, 10).await?;
        wait::for_js_true(
            page,
            &format!("{order} !== {}", serde_json::to_string(&before)?),
            "the column drag to move it",
        )
        .await?;
        Ok("column drag")
    })
    .await
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
                    Ok(row)
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
    const TABLE: &str = "Table full 1000";
    block_on(timed("/table-perf/full", async |page| {
        report(TABLE, &sort_reps(page, "Age").await?, BUDGETS);
        select_reps(page, TABLE).await?;
        let rows = measure(page, REPS, async |_| {
            click_until_changed(
                page,
                "tbody tr:nth-child(2) [data-detail-button]",
                "document.querySelector('tbody tr:nth-child(2) [data-detail-button]').getAttribute('aria-expanded')",
            )
            .await?;
            Ok("detail toggle")
        })
        .await?;
        report(TABLE, &rows, BUDGETS);
        let city = header("City");
        let handle = tag(
            page,
            &format!("{city}.querySelector('[data-resize-handle]')"),
            "resize",
        )
        .await?;
        let width = format!("{city}.style.width");
        let rows = drag_reps(page, &handle, (60.0, 0.0), 10, &width, REPS).await?;
        report(&format!("{TABLE} resize"), &rows, BUDGETS);
        report(TABLE, &column_drag_reps(page).await?, BUDGETS);
        report(TABLE, &filter_keys(page, "ali").await?, BUDGETS);
        Ok(())
    }));
}

/// 5000 rows, 50 a page: next page, and a sort that goes back to page 1.
#[test]
#[ignore = "table timing report, run on request"]
fn paging() {
    const TABLE: &str = "Table paged 5000";
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
            Ok("next page")
        })
        .await?;
        report(TABLE, &rows, BUDGETS);
        page.evaluate("window.scrollTo(0, 0)").await?;
        report(TABLE, &sort_reps(page, "Age").await?, BUDGETS);
        select_reps(page, TABLE).await
    }));
}

/// 10k windowed rows: wheel notches and long jumps, a sort, select-all and filtering.
#[test]
#[ignore = "table timing report, run on request"]
fn a_windowed_table() {
    const TABLE: &str = "Table windowed 10k";
    block_on(timed("/table-perf/windowed", async |page| {
        report(TABLE, &wheel_reps(page, REPS).await?, BUDGETS);
        page.evaluate(format!("{SCROLLER}.scrollTop = 0")).await?;
        let rows = measure(page, REPS, async |_| {
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
            Ok("jump")
        })
        .await?;
        report(TABLE, &rows, BUDGETS);
        page.evaluate(format!("{SCROLLER}.scrollTop = 0")).await?;
        report(TABLE, &sort_reps(page, "Age").await?, BUDGETS);
        report(TABLE, &select_all_reps(page).await?, BUDGETS);
        report(TABLE, &column_drag_reps(page).await?, BUDGETS);
        // Deep in the list, where a reorder was laggy (todo 2584).
        page.evaluate(format!("{SCROLLER}.scrollTop = 200000"))
            .await?;
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('tbody tr')].some((tr) => tr.getAttribute('aria-rowindex') > 5000)",
            "row 5000 in view",
        )
        .await?;
        report(
            &format!("{TABLE} at 5000"),
            &column_drag_reps(page).await?,
            BUDGETS,
        );
        report(TABLE, &filter_keys(page, "ali").await?, BUDGETS);
        Ok(())
    }));
}
