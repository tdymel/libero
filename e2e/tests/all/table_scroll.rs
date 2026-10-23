//! `Table` scrolling: the sticky header, focus that scrolls the rows, and windowed or infinite rows.

use super::table::AREA;
use anyhow::{Result, bail, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, ax, wait};
use std::time::{Duration, Instant};

/// Todo 734: Tab scrolls the region to the last header's button. Blitz scrolled
/// nothing on a focus move, and held the table at the region's width.
async fn the_tabbed_last_header_shows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const LAST: &str = "th:last-child button";
    d.focus("th:first-child button").await?;
    for _ in 0..8 {
        if d.is_focused(LAST).await? {
            break;
        }
        d.press(keyboard::TAB).await?;
    }
    eventually_focused(d, LAST, "Tab").await?;
    eventually(d, "the region to show the last header", async |d| {
        let region = d.rect(AREA).await?;
        let last = d.rect(LAST).await?;
        Ok(last.x + last.width <= region.x + region.width + 1.0 && last.x >= region.x - 1.0)
    })
    .await
}

e2e::scenario!(
    a_tabbed_header_scrolls_into_its_region,
    "/table/wide",
    the_tabbed_last_header_shows,
    android: skip("958: element identity on the WebView"),
    ios: skip("958: element identity on the WebView")
);

/// Todo 3204: a table capped after its mount still measures its header for the scroll padding.
async fn a_cap_added_later_pads_the_scroll_by_the_header<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    // Blitz's stylo lacks `scroll-padding`: it reads the var the Table sets beside it.
    let property = match d.platform() {
        Platform::Native => "--lsx-scroll-padding-top",
        _ => "scroll-padding-top",
    };
    d.click("#cap").await?;
    eventually(
        d,
        "the scroll padding to equal the header's height",
        async |d| {
            if !d.exists(AREA).await? {
                return Ok(false);
            }
            let padding = d.style(AREA, property).await?;
            let padding: f64 = padding.trim().trim_end_matches("px").parse().unwrap_or(0.0);
            let header = d.rect("thead").await?.height;
            Ok(padding > 1.0 && (padding - header).abs() <= 1.0)
        },
    )
    .await
}

e2e::scenario!(
    a_cap_added_later_is_measured,
    "/table/cap-toggle",
    a_cap_added_later_pads_the_scroll_by_the_header
);

/// The header row's top, the first body row's top and the area's top.
async fn tops<D: Driver>(d: &mut D) -> Result<(f64, f64, f64)> {
    Ok((
        d.rect("thead th").await?.y,
        // A `tr` has no box on Blitz.
        d.rect("tbody td").await?.y,
        d.rect(AREA).await?.y,
    ))
}

/// The rows scroll under the header, which stays at the area's top, opaque.
async fn scrolls_under_the_header<D: Driver>(
    d: &mut D,
    key: keyboard::Key,
    presses: usize,
) -> Result<()> {
    let (_, row, _) = tops(d).await?;
    for _ in 1..presses {
        let (_, before, _) = tops(d).await?;
        d.press(key).await?;
        let mut sent = Instant::now();
        eventually(d, "a step to scroll the rows", async |d| {
            if tops(d).await?.1 < before {
                return Ok(true);
            }
            // WKWebView sometimes drops a press: send it again.
            if d.platform() == Platform::Desktop && sent.elapsed() > Duration::from_secs(2) {
                d.press(key).await?;
                sent = Instant::now();
            }
            Ok(false)
        })
        .await?;
        // WebKit drops a press that lands while the last step still animates; Chromium
        // retargets it, and Blitz does not animate.
        if d.platform() != Platform::Desktop {
            continue;
        }
        // Still over three polls in a row.
        let mut last = tops(d).await?.1;
        let mut still = 0;
        eventually(d, "the step's scroll to settle", async |d| {
            let now = tops(d).await?.1;
            still = if now == last { still + 1 } else { 0 };
            last = now;
            Ok(still >= 3)
        })
        .await?;
    }
    d.press(key).await?;
    eventually(d, "the rows to scroll under the header", async |d| {
        let (header, moved, area) = tops(d).await?;
        Ok(moved < row - 20.0 && (header - area).abs() <= 1.0)
    })
    .await?;
    let background = d.style("thead th", "background-color").await?;
    if background == "rgba(0, 0, 0, 0)" || background == "transparent" {
        bail!("the sticky header is see-through: {background}");
    }
    Ok(())
}

/// 1156-2c: with a sort button focused, PageDown scrolls the capped body.
async fn a_focused_header_scrolls_the_rows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("th:first-child [data-sort-button]").await?;
    eventually_focused(d, "th:first-child [data-sort-button]", "focus").await?;
    if d.attr(AREA, "role").await?.is_some()
        || d.attr(AREA, "tabindex").await?.as_deref() == Some("0")
    {
        bail!("the area is a tab stop beside the header buttons");
    }
    scrolls_under_the_header(d, keyboard::PAGE_DOWN, 1).await?;
    // The menu opens in a portal: the 200px area does not clip it.
    const OPTIONS: &str = "button[aria-label=\"Stock column options\"]";
    if d.platform() == Platform::Native {
        // A stuck cell moves by `translate`, which Blitz's hit test ignores: keys, not a click.
        d.focus(OPTIONS).await?;
        d.press(keyboard::ENTER).await?;
    } else {
        d.click(OPTIONS).await?;
    }
    eventually(d, "the column menu to open unclipped", async |d| {
        if !d.exists("[role=menu]").await? || d.exists(&format!("{AREA} [role=menu]")).await? {
            return Ok(false);
        }
        Ok(d.rect("[role=menu]").await?.height > 50.0)
    })
    .await
}

e2e::scenario!(
    a_capped_table_scrolls_from_its_header,
    "/table/sticky",
    a_focused_header_scrolls_the_rows,
    android: skip("958: element identity on the WebView")
);

/// 1156-5a: 10k rows under a 300px cap: only the rows in view render, End brings
/// the last one in under the header, and each row keeps its place in the count.
async fn the_window_follows_the_scroll<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const FIRST: &str = "tr[aria-rowindex=\"2\"]";
    const LAST: &str = "tr[aria-rowindex=\"10001\"]";
    eventually(d, "only the first rows to render", async |d| {
        Ok(
            d.attr("table", "aria-rowcount").await?.as_deref() == Some("10001")
                && d.exists(FIRST).await?
                && !d.exists("tr[aria-rowindex=\"60\"]").await?,
        )
    })
    .await?;
    let row = d.rect("tbody th").await?;
    if (row.height - 40.0).abs() > 1.0 {
        bail!("a row is {}px, not its 40px row_height", row.height);
    }
    const NAME: &str = "thead th:nth-child(2)";
    let width = d.rect(NAME).await?.width;
    d.focus("th[data-sortable] [data-sort-button]").await?;
    d.press(keyboard::END).await?;
    eventually(d, "the last row to show under the header", async |d| {
        if !d.exists(LAST).await? || d.exists(FIRST).await? {
            return Ok(false);
        }
        let (area, header, last) = (
            d.rect(AREA).await?,
            d.rect("thead th").await?,
            d.rect(&format!("{LAST} th")).await?,
        );
        Ok((header.y - area.y).abs() <= 1.0
            && last.y >= header.y + header.height - 1.0
            && last.y + last.height <= area.y + area.height + 1.0)
    })
    .await?;
    // Fixed layout: the longer names down here do not widen their column.
    let scrolled = d.rect(NAME).await?.width;
    if (scrolled - width).abs() > 0.5 {
        bail!("the Name column went from {width}px to {scrolled}px while scrolling");
    }
    d.click(&format!("{LAST} td[data-select] span[aria-hidden]"))
        .await?;
    eventually_text(d, "#selection", "10000", "a click on the last row's box").await?;
    let last_box = format!("{LAST} input");
    // The focused row stays rendered out of view, focus still in it; on Blitz too (todo 1406).
    eventually_focused(d, &last_box, "a click on the last row's box").await?;
    d.press(keyboard::HOME).await?;
    eventually(d, "the first rows to come back", async |d| {
        Ok(d.exists(FIRST).await? && !d.exists("tr[aria-rowindex=\"9990\"]").await?)
    })
    .await?;
    eventually_focused(d, &last_box, "Home").await?;
    let first_box = format!("{FIRST} input");
    d.click(&format!("{FIRST} td[data-select] span[aria-hidden]"))
        .await?;
    eventually_focused(d, &first_box, "a click on the first row's box").await?;
    d.press(keyboard::END).await?;
    eventually(d, "the last rows to come back", async |d| {
        Ok(d.exists(LAST).await? && !d.exists("tr[aria-rowindex=\"10\"]").await?)
    })
    .await?;
    eventually_focused(d, &first_box, "End").await
}

e2e::scenario!(
    a_windowed_table_renders_the_rows_in_view,
    "/table/windowed",
    the_window_follows_the_scroll,
    ios: skip("3633: End reaches the page but scrolls no region, as in Safari")
);

/// 1156-5c: Tab walks the row checkboxes past the rendered rows and back: the
/// focused box scrolls into view, and the window renders the next row in time.
async fn tab_walks_past_the_window<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const STEPS: usize = 30;
    let at = |index: usize| format!("tr[aria-rowindex=\"{index}\"] input");
    eventually(d, "the first row", async |d| d.exists(&at(2)).await).await?;
    d.focus(&at(2)).await?;
    // At a person's pace: the window settles after each scroll before the next key.
    for step in 1..=STEPS {
        let next = at(2 + step);
        eventually(d, &format!("{next} rendered before Tab"), async |d| {
            d.exists(&next).await
        })
        .await?;
        d.press(keyboard::TAB).await?;
        eventually_focused(d, &next, &format!("Tab {step}")).await?;
    }
    for step in (0..STEPS).rev() {
        let previous = at(2 + step);
        eventually(
            d,
            &format!("{previous} rendered before Shift+Tab"),
            async |d| d.exists(&previous).await,
        )
        .await?;
        d.press_shift(keyboard::TAB).await?;
        eventually_focused(d, &previous, "Shift+Tab").await?;
    }
    Ok(())
}

e2e::scenario!(
    tab_walks_the_rows_of_a_windowed_table,
    "/table/windowed",
    tab_walks_past_the_window
);

/// 1156-5d: End at the bottom asks for the next batch, each End once, and the
/// count of rows that arrived is said; past the last batch nothing more comes.
async fn end_loads_the_next_rows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const LIVE: &str = "table + [role=status]";
    row_count_is(d, "51").await?;
    d.focus(AREA).await?;
    for (asks, rows, said) in [("1", "101", "100 rows"), ("2", "151", "150 rows")] {
        d.press(keyboard::END).await?;
        eventually_text(d, "#asks", asks, "End at the bottom").await?;
        row_count_is(d, rows).await?;
        eventually_text(d, LIVE, said, "the rows that arrived").await?;
    }
    d.press(keyboard::END).await?;
    eventually_text(d, "#asks", "3", "End past the last batch").await?;
    // Rendered with the third ask: no batch on its way, so no row can arrive (was a 1 s wait).
    if d.attr("#asks", "data-loading").await?.as_deref() != Some("false") {
        bail!("End past the last batch started loading another batch");
    }
    row_count_is(d, "151").await
}

e2e::scenario!(
    end_at_the_bottom_loads_more_rows,
    "/table/infinite",
    end_loads_the_next_rows,
    ios: skip("3633: End reaches the page but scrolls no region, as in Safari")
);

async fn row_count_is<D: Driver>(d: &mut D, rows: &str) -> Result<()> {
    eventually(d, &format!("aria-rowcount {rows}"), async |d| {
        Ok(d.attr("table", "aria-rowcount").await?.as_deref() == Some(rows))
    })
    .await
}

/// 1156-2c: no button inside, so the overflowing area is the named tab stop.
async fn a_plain_capped_table_is_a_named_stop<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // 1313: a WebView reads the inner focusables by the area's tag.
    eventually(d, "the area to become a region", async |d| {
        Ok(d.attr(AREA, "role").await?.as_deref() == Some("region")
            && d.attr(AREA, "tabindex").await?.as_deref() == Some("0"))
    })
    .await?;
    // Blitz draws the caption as a div before the table, under the same id.
    let name = d.attr(AREA, "aria-labelledby").await?.unwrap_or_default();
    if d.text(&format!("[id=\"{name}\"]")).await? != "Fruit stock" {
        bail!("the area is not named by the caption: {name:?}");
    }
    d.focus(AREA).await?;
    eventually_focused(d, AREA, "focus").await?;
    // The caption scrolls first: one WebKit arrow step (34px) leaves the header below it.
    scrolls_under_the_header(d, keyboard::ARROW_DOWN, 2).await
}

e2e::scenario!(
    a_plain_capped_table_scrolls_as_a_named_region,
    "/table/sticky-plain",
    a_plain_capped_table_is_a_named_stop,
    android: skip("958: element identity on the WebView")
);

/// Todo 1455: scrolled step by step, the rendered rows always cover the area under the header.
#[test]
fn a_windowed_table_has_no_blank_rows_while_scrolling() {
    block_on(async {
        let fixture = Fixture::open("/table/windowed", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "!!document.querySelector('tr[aria-rowindex=\"2\"]')",
            "the first rows",
        )
        .await
        .unwrap();
        // The px of the area under the header that no rendered row covers. The sticky
        // cells, not the `thead`: it scrolls away above the reserved rows (2176).
        const GAP: &str = "(() => { const area = document.querySelector('[data-table-scroll]'); \
             const a = area.getBoundingClientRect(); \
             const top = document.querySelector('thead th').getBoundingClientRect().bottom; \
             const bottom = a.top + area.clientHeight; \
             const rows = [...document.querySelectorAll('tbody tr')].map(r => r.getBoundingClientRect()); \
             const first = Math.min(...rows.map(r => r.top)), last = Math.max(...rows.map(r => r.bottom)); \
             return Math.max(0, first - top) + Math.max(0, bottom - last); })()";
        // Firefox anchored on the drawn scrollbar layer and scrolled on after it, to the bottom.
        let anchor: String = page
            .evaluate_bounded(
                "getComputedStyle(document.querySelector('[data-table-scroll] > [data-slot=scrollbars]')).overflowAnchor",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            anchor, "none",
            "the drawn scrollbar layer can be a scroll anchor"
        );
        // Todo 1454: the scrollbar runs beside the rows, not over the header.
        let bar: String = page
            .evaluate_bounded(
                "(() => { const bar = document.querySelector('[data-table-scroll] [data-orientation=vertical]'); \
                 const head = document.querySelector('thead').getBoundingClientRect(); \
                 const area = document.querySelector('[data-table-scroll]').getBoundingClientRect(); \
                 const b = bar.getBoundingClientRect(); \
                 return [Math.abs(b.top - head.bottom) <= 1, Math.abs(b.bottom - area.bottom) <= 1].join('|') \
                   + ' ' + JSON.stringify([b, head, area]); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            bar.starts_with("true|true "),
            "the scrollbar spans the header: {bar}"
        );
        let mut report = Vec::new();
        for step in 1..=20 {
            let at = step * 150;
            page.evaluate_bounded(format!("document.querySelector({AREA:?}).scrollTop = {at}"))
                .await
                .unwrap();
            let settled =
                wait::for_js_true(page, &format!("{GAP} < 1"), "the rows to cover the area").await;
            let (gap, top): (f64, f64) = page
                .evaluate_bounded(format!(
                    "[{GAP}, document.querySelector({AREA:?}).scrollTop]"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            if settled.is_err() || (top - at as f64).abs() > 1.0 {
                report.push(format!("at {at}px: {gap}px blank, scrolled on to {top}px"));
            }
        }
        assert!(report.is_empty(), "{report:?}");
        fixture
            .console
            .assert_clean("scrolling a windowed table")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1156-5a: a windowed row is keyed by `row_key`, not its place, so a sort moves its node.
#[test]
fn a_windowed_row_keeps_its_node_through_a_sort() {
    block_on(async {
        let fixture = Fixture::open("/table/windowed", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        const ROW: &str = "[...document.querySelectorAll('tbody tr')]\
                           .find(row => row.querySelector('th').textContent === '4')";

        wait::for_js_true(page, &format!("!!{ROW}"), "the row of stock 4")
            .await
            .unwrap();
        page.evaluate_bounded(format!("{ROW}.marked = true"))
            .await
            .unwrap();
        pointer::click(page, "th[data-sortable] [data-sort-button]")
            .await
            .unwrap();
        // Name ascending: the Apples (4, 8, ...) on top, in source order.
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('tr[aria-rowindex=\"2\"] th').textContent === '4' \
                 && {ROW}.marked === true"
            ),
            "stock 4 on top, on its own node",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("sorting a windowed table")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1420: held Tab and Shift+Tab walk the row checkboxes one by one; the window
/// never falls behind, so focus never leaves the body.
#[test]
fn a_held_tab_walks_a_windowed_body_row_by_row() {
    block_on(async {
        let fixture = Fixture::open("/table/windowed", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        const FOCUSED_ROW: &str = "(() => { const row = document.activeElement.closest('tbody tr'); \
             return row ? row.getAttribute('aria-rowindex') : String(document.activeElement.tagName); })()";
        wait::for_js_true(
            page,
            "(() => { const box = document.querySelector('tr[aria-rowindex=\"2\"] input[type=checkbox]'); \
             if (!box) return false; box.focus(); return document.activeElement === box; })()",
            "the first row's checkbox focused",
        )
        .await
        .unwrap();
        // Bursts of back-to-back presses; after each, the window keeps 10 rows ahead of
        // the focus, past the default overscan of 4, for the next burst to land in.
        const BURST: usize = 10;
        const BURSTS: usize = 6;
        for (modifiers, step) in [(0, BURST as i64), (keyboard::SHIFT, -(BURST as i64))] {
            for _ in 0..BURSTS {
                let from: String = page
                    .evaluate_bounded(FOCUSED_ROW)
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                let expected = from.parse::<i64>().unwrap() + step;
                keyboard::hold(page, keyboard::TAB, modifiers, BURST)
                    .await
                    .unwrap();
                let at = format!("{FOCUSED_ROW} === '{expected}'");
                if wait::for_js_true(page, &at, "focus on the expected row")
                    .await
                    .is_err()
                {
                    let landed: String = page
                        .evaluate_bounded(FOCUSED_ROW)
                        .await
                        .unwrap()
                        .into_value()
                        .unwrap();
                    panic!(
                        "held Tab (modifiers {modifiers}) landed on {landed}, not row {expected}"
                    );
                }
                let caught_up = format!(
                    "(() => {{ const rows = [...document.querySelectorAll('tbody tr[aria-rowindex]')] \
                     .map(row => Number(row.getAttribute('aria-rowindex'))); \
                     return {step} > 0 ? Math.max(...rows) >= {expected} + {step} \
                     : Math.min(...rows) <= Math.max(2, {expected} + {step}); }})()"
                );
                wait::for_js_true(page, &caught_up, "the window caught up with the focus")
                    .await
                    .unwrap();
            }
        }
        fixture
            .console
            .assert_clean("holding Tab in a windowed table")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1156-5c: a screen reader in the middle of a windowed table meets one table:
/// the header row, then each rendered row placed in the whole count, no spacer.
#[test]
fn a_windowed_table_reads_as_the_whole_table() {
    block_on(async {
        let fixture = Fixture::open("/table/windowed", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const r = document.querySelector({AREA:?}); \
                 if (!r) return false; r.scrollTop = r.scrollHeight; \
                 return !!document.querySelector('tr[aria-rowindex=\"10001\"]'); }})()"
            ),
            "the window at the last row",
        )
        .await
        .unwrap();
        // Unsorted, a row's stock is its place in `data`: its row index less the header row.
        let placed: String = page
            .evaluate_bounded(
                "(() => { const rows = [...document.querySelectorAll('table tr')]; \
                 const at = rows.map(r => +r.getAttribute('aria-rowindex')); \
                 const count = +document.querySelector('table').getAttribute('aria-rowcount'); \
                 const bad = rows.slice(1).filter((r, i) => (i > 0 && at[i + 1] !== at[i] + 1) \
                   || r.querySelector('th').textContent !== String(at[i + 1] - 1)); \
                 return [count, at[0], rows.length, at[rows.length - 1], bad.length].join(' '); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [count, head, rendered, last, bad] = placed
            .split(' ')
            .map(|n| n.parse::<usize>().unwrap())
            .collect::<Vec<_>>()[..]
        else {
            panic!("{placed}");
        };
        assert_eq!((count, head, last, bad), (10_001, 1, 10_001, 0), "{placed}");
        assert!(rendered < 40, "{rendered} rows rendered");

        let tree = ax::snapshot(page, "table").await.unwrap();
        assert!(tree.starts_with("table \"Fruit stock\""), "{tree}");
        let rows = tree
            .lines()
            .filter(|line| line.split_whitespace().next() == Some("row"))
            .count();
        // Each `tr` is a row to a screen reader, the spacer padding none.
        assert_eq!(rows, rendered, "{tree}");
        assert!(tree.contains("rowheader \"10000\""), "{tree}");
        assert!(tree.contains("checkbox \"Select 10000\""), "{tree}");

        fixture
            .console
            .assert_clean("reading a windowed table")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A wide table scrolls in its `ScrollArea` (588), which ArrowRight scrolls from a
/// focused header button (654); the buttons make the area no stop (1156-2c).
#[test]
fn a_wide_table_scrolls_from_a_focused_header() {
    const REGION: &str = AREA;
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let at = viewport.name();
            let fixture = Fixture::open("/table/wide", viewport).await.unwrap();
            let page = &fixture.page;

            let state: String = page
                .evaluate_bounded(format!(
                    "(() => {{ const r = document.querySelector({REGION:?}); \
                     const c = document.querySelector('caption'); \
                     return [r.scrollWidth > r.clientWidth, \
                     document.documentElement.scrollWidth <= innerWidth, \
                     r.getAttribute('role'), r.tabIndex, \
                     c.textContent].join('|'); }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(
                state, "true|true||-1|Fruit catalogue",
                "at {at}: overflows itself, not the page; no stop beside its header buttons"
            );

            page.evaluate_bounded("document.querySelector('th button').focus()")
                .await
                .unwrap();
            keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
            wait::for_js_true(
                page,
                &format!("document.querySelector({REGION:?}).scrollLeft > 0"),
                &format!("ArrowRight to scroll the focused region at {at}"),
            )
            .await
            .unwrap();

            fixture
                .console
                .assert_clean("scrolling a wide table")
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Todo 2014: on a phone a windowed table of unsized columns scrolls sideways, its
/// columns kept readable, as an unwindowed one does.
#[test]
fn a_narrow_windowed_table_scrolls_instead_of_squeezing() {
    block_on(async {
        let fixture = Fixture::open("/table/windowed-wide", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "!!document.querySelector('tbody tr[aria-rowindex=\"2\"]')",
            "the first windowed row",
        )
        .await
        .unwrap();
        let state: String = page
            .evaluate_bounded(format!(
                "(() => {{ const area = document.querySelector({AREA:?}); \
                 const cells = [...document.querySelectorAll('thead th')].map(th => th.getBoundingClientRect().width); \
                 const row = document.querySelector('tbody tr[aria-rowindex=\"2\"] td').getBoundingClientRect().width; \
                 return [area.scrollWidth > area.clientWidth + 100, Math.min(...cells) >= 120, row >= 120].join('|') \
                   + ' ' + JSON.stringify([area.scrollWidth, area.clientWidth, cells, row]); }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            state.starts_with("true|true|true "),
            "the area scrolls sideways, every column at least 120px: {state}"
        );
        fixture
            .console
            .assert_clean("a narrow windowed table")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 2.4.11: Shift+Tab up a capped table's rows never leaves the focused
/// checkbox under the sticky header.
async fn a_focused_row_stays_below_the_header<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // No wait for the header: its height is read at mount, not only by its late
    // first resize report (todo 1523).
    d.focus("tbody tr:nth-child(1) input").await?;
    for _ in 0..30 {
        d.press(keyboard::TAB).await?;
        d.settle().await?;
    }
    // Each step polls, as the page may scroll a frame after the focus.
    for step in 0..30 {
        d.press_shift(keyboard::TAB).await?;
        eventually(
            d,
            &format!("the focused row control below the sticky header, Shift+Tab {step}"),
            async |d| {
                let header = d.rect("thead th").await?;
                Ok(d.rect("input:focus").await?.y - (header.y + header.height) >= -1.0)
            },
        )
        .await?;
    }
    Ok(())
}

e2e::scenario!(
    a_focused_row_control_is_not_hidden_by_the_sticky_header,
    "/table/sticky-select",
    a_focused_row_stays_below_the_header,
    android: skip("958: element identity on the WebView")
);

/// Todo 3968: `sticky_header` holds the header at its offset while the page, or the `#area`
/// around the table, scrolls past the rows, and a focus scroll leaves a row clear of it.
async fn the_header_holds_at_its_offset_under_the_scroll<D: Driver>(
    d: &mut D,
    route: &str,
) -> Result<()> {
    const OFFSET: f64 = 40.0;
    let boxed = route.ends_with("area-sticky");
    let origin = match boxed {
        true => d.rect("#area").await?.y,
        false => 0.0,
    };
    let at_rest = d.rect("thead th").await?.y - origin;
    ensure!(
        at_rest > 200.0,
        "the header starts below the banner: {at_rest}"
    );
    match boxed {
        true => d.scroll_to("#area", 0.0, 900.0).await?,
        false => d.scroll_by(900.0).await?,
    }
    eventually(d, "the header held at its offset", async |d| {
        Ok((d.rect("thead th").await?.y - origin - OFFSET).abs() <= 1.0)
    })
    .await?;
    let header = d.rect("thead th").await?;
    let row = d.rect("tbody tr:nth-child(30)").await?;
    ensure!(
        row.y + row.height <= header.y || row.y >= header.y + header.height - 1.0,
        "a row paints under the header: {row:?} vs {header:?}"
    );
    // Blitz reads no `scroll-margin-top` back.
    if !matches!(d.platform(), Platform::Native) {
        let margin = d.style("tbody tr", "scroll-margin-top").await?;
        let margin: f64 = margin.trim_end_matches("px").parse()?;
        ensure!(
            (margin - (OFFSET + header.height)).abs() <= 1.0,
            "scroll margin {margin} is not the offset plus the header's {}",
            header.height
        );
    }
    Ok(())
}

e2e::scenario!(
    a_page_sticky_header_holds_at_its_offset,
    "/table/page-sticky",
    the_header_holds_at_its_offset_under_the_scroll
);

e2e::scenario!(
    a_sticky_header_holds_at_its_offset_in_a_scroll_area,
    "/table/area-sticky",
    the_header_holds_at_its_offset_under_the_scroll
);
