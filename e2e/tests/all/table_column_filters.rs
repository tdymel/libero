//! `Table` filters: the quick filter, the header and menu filters, the filter panel and the date filter.

use super::table::{PICKER, menu_entry, pick_later};
use anyhow::{Result, bail};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

/// 1156-3a: typing narrows the rows, goes back to page 1 and, once settled,
/// announces the count; the unsearched stock column matches nothing.
async fn the_quick_filter_narrows_the_rows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const RANGE: &str = "[data-slot=range]";
    const LIVE: &str = "table + [role=status]";
    eventually_text(d, RANGE, "1–3 of 7", "the first page").await?;
    d.click("button[aria-label=\"Go to next page\"]").await?;
    eventually_text(d, "#page", "2", "Next").await?;
    d.click("input[type=search]").await?;
    eventually_focused(d, "input[type=search]", "a click").await?;
    d.type_text("ap").await?;
    eventually_text(d, "#query", "ap", "typing ap").await?;
    eventually_text(d, RANGE, "1–2 of 2", "the filter").await?;
    eventually_text(d, "tbody th", "Apple", "the filter").await?;
    eventually_text(d, "#page", "1", "the filter's page reset").await?;
    eventually_text(d, LIVE, "2 rows", "the settled count").await?;
    d.press(keyboard::BACKSPACE).await?;
    d.press(keyboard::BACKSPACE).await?;
    d.type_text("7").await?;
    eventually_text(d, "tbody td", "No matching rows", "a stock number").await
}

e2e::scenario!(
    the_quick_filter_narrows_a_paged_table,
    "/table/filter",
    the_quick_filter_narrows_the_rows,
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);

/// 1156-3b/3c: a header filter narrows by value once typing settles; the menu's
/// Filter opens a dialog on the value field, whose operator and Clear apply at once.
async fn the_column_filters_narrow_the_rows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const LIVE: &str = "table + [role=status]";
    const STOCK: &str = "input[aria-label=\"Filter Stock\"]";
    const POPOVER: &str = "[data-filter-popover]";
    const MENU: &str = "[aria-label=\"Name column options\"]";
    d.click(STOCK).await?;
    eventually_focused(d, STOCK, "a click").await?;
    d.type_text("3").await?;
    eventually_text(d, "#filters", "Stock Equals 3", "a header filter").await?;
    eventually_text(d, "tbody th", "Grape", "Stock = 3").await?;
    eventually_text(d, LIVE, "1 row", "the settled count").await?;
    d.press(keyboard::BACKSPACE).await?;
    eventually_text(d, "tbody th", "Fig", "an emptied header filter").await?;

    // Filter follows the sort entries: first, as Name does not sort.
    d.focus(MENU).await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, "[role=menuitem]", "the opened column menu").await?;
    d.press(keyboard::HOME).await?;
    eventually_text(d, "[role=menuitem]:focus", "Filter", "Home").await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "the opened filter", async |d| d.exists(POPOVER).await).await?;
    eventually_focused(d, &format!("{POPOVER} input"), "the opened filter").await?;
    d.type_text("an").await?;
    eventually_text(d, "tbody th", "Banana", "Name contains an").await?;
    eventually(d, "the filtered button", async |d| {
        d.exists("[data-filtered]").await
    })
    .await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, &format!("{POPOVER} {PICKER}"), "Shift+Tab").await?;
    pick_later(d, 1).await?;
    eventually_text(d, "#filters", "Name DoesNotContain an", "the next operator").await?;
    eventually_text(d, "tbody th", "Fig", "Name lacks an").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the closed filter", async |d| {
        Ok(!d.exists(POPOVER).await?)
    })
    .await?;
    eventually_focused(d, MENU, "Escape").await?;

    // Todo 3221: Escape returns to the filtered button that opened it, not the menu button.
    d.click("[data-filtered]").await?;
    eventually(d, "the reopened filter", async |d| d.exists(POPOVER).await).await?;
    eventually_focused(d, &format!("{POPOVER} input"), "the reopened filter").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the closed reopened filter", async |d| {
        Ok(!d.exists(POPOVER).await?)
    })
    .await?;
    eventually_focused(d, "[data-filtered]", "Escape from the filtered button").await?;

    d.click("[data-filtered]").await?;
    eventually(d, "the reopened filter", async |d| d.exists(POPOVER).await).await?;
    d.click(&format!("{POPOVER} button")).await?;
    eventually(d, "a cleared filter", async |d| {
        Ok(!d.exists("[data-filtered]").await?)
    })
    .await?;
    eventually_text(d, "#filters", "", "Clear").await
}

e2e::scenario!(
    the_column_filters_narrow_a_table,
    "/table/column-filter",
    the_column_filters_narrow_the_rows,
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);

/// Todo 1403: Tab past the filter dialog's last stop and Shift+Tab past its first close it,
/// the Tab on to the stop after the menu button.
async fn tab_leaves_the_filter<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const POPOVER: &str = "[data-filter-popover]";
    const MENU: &str = "[aria-label=\"Name column options\"]";
    async fn open<D: Driver>(d: &mut D) -> Result<()> {
        d.focus(MENU).await?;
        d.press(keyboard::ENTER).await?;
        eventually_focused(d, "[role=menuitem]", "the opened column menu").await?;
        d.press(keyboard::HOME).await?;
        eventually_text(d, "[role=menuitem]:focus", "Filter", "Home").await?;
        d.press(keyboard::ENTER).await?;
        eventually_focused(d, &format!("{POPOVER} input"), "the opened filter").await
    }
    // Clear is disabled without a filter: the value field is the last stop.
    open(d).await?;
    d.press(keyboard::TAB).await?;
    eventually(d, "the filter closed by Tab", async |d| {
        Ok(!d.exists(POPOVER).await? && !d.is_focused(MENU).await?)
    })
    .await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, MENU, "Shift+Tab back to the menu button").await?;

    open(d).await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(
        d,
        &format!("{POPOVER} {PICKER}"),
        "Shift+Tab to the operator",
    )
    .await?;
    d.press_shift(keyboard::TAB).await?;
    eventually(d, "the filter closed by Shift+Tab", async |d| {
        Ok(!d.exists(POPOVER).await?)
    })
    .await?;
    eventually_focused(d, MENU, "Shift+Tab past the first stop").await
}

e2e::scenario!(
    tab_past_either_end_of_a_column_filter_closes_it,
    "/table/column-filter",
    tab_leaves_the_filter,
    ios: skip("3416: focus stays on the body after a Tab or a tap")
);

/// Todo 2048, 1.4.10: in a short viewport the filter popover caps at the room on its
/// side and scrolls inside, rather than running off the screen.
#[test]
fn a_filter_popover_caps_at_the_available_height() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    block_on(async {
        let fixture = Fixture::open("/table/column-filter", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let (width, _) = Viewport::Desktop.size();
        page.execute(SetDeviceMetricsOverrideParams::new(width, 200, 1.0, false))
            .await
            .unwrap();
        page.evaluate("document.querySelector('[aria-label=\"Name column options\"]').focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.matches('[role=menuitem]')",
            "the opened column menu",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::HOME).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.textContent.trim() === 'Filter'",
            "Home on Filter",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        // Capped (it scrolls, so the viewport is short enough to matter) and inside.
        let state = "(() => { const p = document.querySelector('[data-filter-popover]'); \
             if (!p) return 'closed'; const r = p.getBoundingClientRect(); \
             const ok = getComputedStyle(p).visibility !== 'hidden' && r.top >= -1 \
               && r.bottom <= innerHeight + 1 && p.scrollHeight > p.clientHeight + 1 \
               && getComputedStyle(p).overflowY === 'auto'; \
             return ok ? 'capped' : JSON.stringify([r.top, r.bottom, innerHeight, p.scrollHeight, \
               p.clientHeight, getComputedStyle(p).maxHeight]); })()";
        let capped =
            wait::for_js_true(page, &format!("{state} === 'capped'"), "the capped popover").await;
        if capped.is_err() {
            let state: String = page.evaluate(state).await.unwrap().into_value().unwrap();
            panic!("the filter popover is not capped inside a 200px viewport: {state}");
        }
        fixture
            .console
            .assert_clean("a filter popover in a short viewport")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1401: a date header filter keeps the typed day; the menu's Filter set to
/// Between keeps the days from its From field on, the To field left open.
/// 1425: a `DateField`, whose calendar counts as inside the filter popover; a
/// WebView keeps its native date input.
async fn a_date_filter_narrows_the_rows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const DUE: &str = "input[aria-label=\"Filter Due\"]";
    const POPOVER: &str = "[data-filter-popover]";
    const CALENDAR: &str = "[role=dialog]:not([data-filter-popover])";
    let webview = matches!(d.platform(), Platform::Android | Platform::Desktop);
    let native_input = d.attr(DUE, "type").await?.as_deref() == Some("date");
    if native_input != webview {
        bail!(
            "{:?}: the Due header filter's type=date is {native_input}",
            d.platform()
        );
    }
    // WebKitGTK's click lands on the segment under it; focus starts at the month.
    match d.platform() {
        Platform::Desktop => d.focus(DUE).await?,
        _ => d.click(DUE).await?,
    }
    match native_input {
        // WebKitGTK's en-US segments advance only on a separator; Chromium's on a full one.
        true if d.platform() == Platform::Desktop => d.type_text("03/10/2024").await?,
        // Chromium's en-US date input takes month, day, then year.
        true => d.type_text("03102024").await?,
        false => {
            d.type_text("March 10, 2024").await?;
            d.press(keyboard::ENTER).await?;
            d.press(keyboard::ESCAPE).await?;
            eventually(
                d,
                "Escape to close the header filter's calendar",
                async |d| Ok(!d.exists(CALENDAR).await?),
            )
            .await?;
        }
    }
    eventually_text(d, "#filters", "Equals 2024-03-10..", "a typed day").await?;
    eventually_text(d, "tbody th", "Grape", "Due on the 10th").await?;

    d.focus("[aria-label=\"Due column options\"]").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, "[role=menuitem]", "the opened column menu").await?;
    d.press(keyboard::HOME).await?;
    eventually_text(d, "[role=menuitem]:focus", "Filter", "Home").await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "the opened filter", async |d| d.exists(POPOVER).await).await?;
    // WebView2 lands the focus a task after the popover mounts: Shift+Tab waits for it.
    let value = format!("{POPOVER} input[data-filter-value]");
    eventually_focused(d, &value, "the opened filter").await?;
    if !native_input {
        // Into the value field's calendar and back: the popover stays open throughout.
        d.press(keyboard::ARROW_DOWN).await?;
        eventually_focused(
            d,
            &format!("{CALENDAR} [data-slot=day][tabindex=\"0\"]"),
            "Arrow Down into the calendar",
        )
        .await?;
        // A close on that focus would land a task later, long before Escape's round trip.
        d.press(keyboard::ESCAPE).await?;
        eventually_focused(d, &value, "Escape").await?;
        eventually(d, "Escape to close the calendar", async |d| {
            Ok(!d.exists(CALENDAR).await?)
        })
        .await?;
        if !d.exists(POPOVER).await? {
            bail!(
                "{:?}: focus in the calendar or its Escape closed the filter",
                d.platform()
            );
        }
    }
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, &format!("{POPOVER} {PICKER}"), "Shift+Tab").await?;
    // Equals, then Does not equal, Before, After, On or before, On or after, Between.
    pick_later(d, 6).await?;
    eventually_text(
        d,
        "#filters",
        "Between 2024-03-10..",
        "Between from the 10th",
    )
    .await?;
    eventually(d, "the 10th, 15th and 20th", async |d| {
        Ok(d.exists("tbody tr:nth-child(3)").await?
            && !d.exists("tbody tr:nth-child(4)").await?
            && d.text("tbody tr:first-child th").await? == "Grape")
    })
    .await?;
    let fields = d.text(POPOVER).await?;
    if !(fields.contains("From") && fields.contains("To")) {
        bail!("Between shows no From and To fields: {fields:?}");
    }
    Ok(())
}

e2e::scenario!(
    a_date_filter_narrows_a_table,
    "/table/date-filter",
    a_date_filter_narrows_the_rows,
    android: skip("Chromium's typed date segments only"),
    windows: skip("3422: Shift+Tab never leaves the date input on WebView2"),
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);

/// 1400: the filter panel adds lines, joins them by the logic, and moves the
/// focus as Linus set: Add to the new line, remove to the previous one,
/// close to the Filters button; a column menu's Filter opens it on its line.
async fn the_filter_panel_edits_every_filter<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const BUTTON: &str = "[data-filter-panel-button]";
    const PANEL: &str = "[data-filter-panel]";
    const LINE_0: &str = "[data-filter-line=\"0\"]";
    const LINE_1: &str = "[data-filter-line=\"1\"]";
    d.click(BUTTON).await?;
    eventually(d, "the opened panel", async |d| d.exists(PANEL).await).await?;
    eventually_focused(d, "[data-filter-add]", "an empty panel").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, &format!("{LINE_0} {PICKER}"), "Add").await?;
    eventually_text(d, "#filters", "Name Contains ", "the added line").await?;
    d.click(&format!("{LINE_0} input")).await?;
    d.type_text("e").await?;
    eventually_text(d, "#filters", "Name Contains e", "typing in a line").await?;

    // Add takes the first column without a line: Stock.
    d.click("[data-filter-add]").await?;
    eventually_focused(d, &format!("{LINE_1} {PICKER}"), "a second Add").await?;
    d.click(&format!("{LINE_1} input")).await?;
    d.type_text("1").await?;
    eventually_text(
        d,
        "#filters",
        "Name Contains e; Stock Equals 1",
        "a second line",
    )
    .await?;
    // Names with an e and a stock of 1: none.
    eventually(d, "no rows under And", async |d| {
        Ok(!d.exists("tbody th").await?)
    })
    .await?;
    // The first: the logic sits above the lines.
    d.focus(&format!("{PANEL} {PICKER}")).await?;
    pick_later(d, 1).await?;
    eventually_text(d, "#logic", "Or", "the logic pick").await?;
    eventually_text(
        d,
        "tbody tr:first-child th",
        "Fig",
        "Or keeps Fig by its stock",
    )
    .await?;

    d.click(&format!("{LINE_1} [data-filter-remove]")).await?;
    eventually_text(d, "#filters", "Name Contains e", "a removed line").await?;
    eventually_focused(d, &format!("{LINE_0} {PICKER}"), "the remove").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the closed panel", async |d| {
        Ok(!d.exists(PANEL).await?)
    })
    .await?;
    eventually_focused(d, BUTTON, "Escape").await?;
    if d.attr(BUTTON, "aria-label").await?.as_deref() != Some("Filters, 1 active") {
        bail!("the button does not name the active count");
    }

    // No settle: the closed panel's late focus-out check took focus from the reopened one (1656).
    d.click("[aria-label=\"Stock column options\"]").await?;
    // Kept from the read that found it, not read again (1675).
    let item = menu_entry(d, "Filter").await?;
    d.click(&item).await?;
    eventually(d, "the panel from the menu", async |d| {
        d.exists(PANEL).await
    })
    .await?;
    eventually_focused(d, &format!("{LINE_1} {PICKER}"), "the menu's Filter").await?;
    // A real press outside still closes the reopened panel.
    d.click("#filters").await?;
    eventually(d, "the panel closed by a press outside", async |d| {
        Ok(!d.exists(PANEL).await?)
    })
    .await
}

e2e::scenario!(
    the_filter_panel_edits_every_filter,
    "/table/filter-panel",
    the_filter_panel_edits_every_filter,
    android: skip("958: element identity on the WebView"),
    ios: skip("958: element identity on the WebView")
);
