//! `Table`: a header click sorts, a second flips it, a third unsorts, and a custom cell body
//! moves with its row. Plain cells draw their text inline (todo 29).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text, linger};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, ax, wait};

const SORT: &str = "th[data-sortable] button";

async fn sorted<D: Driver>(d: &mut D, expected: &str) -> Result<()> {
    eventually(d, &format!("aria-sort {expected}"), async |d| {
        Ok(d.attr("th[data-sortable]", "aria-sort").await?.as_deref() == Some(expected))
    })
    .await
}

async fn the_arrow_turns<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const ARROW: &str = "th[data-sortable] svg";
    d.click(SORT).await?;
    sorted(d, "ascending").await?;
    // Past the turn's transition.
    linger(d, 30).await;
    let ascending = d.style(ARROW, "transform").await?;
    d.click(SORT).await?;
    sorted(d, "descending").await?;
    eventually(
        d,
        &format!("the arrow to turn from {ascending}"),
        async |d| Ok(d.style(ARROW, "transform").await? != ascending),
    )
    .await
}

e2e::scenario!(
    the_arrow_turns_when_the_sort_flips,
    "/table",
    the_arrow_turns
);

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
    android: skip("958: element identity on the WebView")
);

const AREA: &str = "[data-table-scroll]";

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
        // WebKit drops a press that lands while the last step still animates.
        let (_, before, _) = tops(d).await?;
        d.press(key).await?;
        eventually(d, "a step to scroll the rows", async |d| {
            Ok(tops(d).await?.1 < before)
        })
        .await?;
        linger(d, 300).await;
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

/// 1156-0b/0c: a row click reports its row; states, stripes, `sm` padding and
/// a formatted cell all show.
async fn the_rows_report_and_show_their_look<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const APPLE: &str = "tr[data-name=\"Apple\"]";
    // Before any click, so no row is hovered: the hover wins over a stripe.
    let clear = d.style("tbody tr:nth-child(1)", "background-color").await?;
    let stripe = d.style("tbody tr:nth-child(2)", "background-color").await?;
    if clear == stripe {
        bail!("the second row is not striped: {stripe}");
    }
    // 1156 follow-up: a hovered stripe takes its own shade.
    d.hover("tbody tr:nth-child(2) td").await?;
    eventually(d, "a hovered striped row to change", async |d| {
        let hovered = d.style("tbody tr:nth-child(2)", "background-color").await?;
        Ok(hovered != stripe && hovered != clear)
    })
    .await?;
    let padding = d.style("tbody td", "padding-top").await?;
    if padding != "6px" {
        bail!("an sm cell's padding is {padding}");
    }
    let color = d.style("tr[data-state~=\"sold-out\"] td", "color").await?;
    if color != "rgb(255, 0, 0)" {
        bail!("the sold-out row's state did not style it: {color}");
    }
    eventually_text(d, &format!("{APPLE} td"), "12 kg", "formatting").await?;
    d.click(&format!("{APPLE} td")).await?;
    eventually_text(d, "#clicked", "Apple", "a row click").await
}

e2e::scenario!(
    a_row_click_reports_its_row,
    "/table/rows",
    the_rows_report_and_show_their_look
);

/// The checkbox box of body row `n` (1-based), or of the header with 0.
fn select_box(n: usize) -> String {
    match n {
        0 => "thead th[data-select] span[aria-hidden]".to_string(),
        n => format!("tbody tr:nth-child({n}) td[data-select] span[aria-hidden]"),
    }
}

const STOCK: &str = "th[data-sortable]:nth-child(3) button";
const NAME: &str = "th[data-sortable]:nth-child(2) button";

/// 1156-1a/1c: a row's box selects it without a row click, select-all mixes and
/// fills, the selection follows its rows through a sort; a tap on touch adds a
/// sorted column, a click on a pointer replaces it.
async fn rows_select_and_sort<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#selection", "Apple", "the seeded selection").await?;
    if d.attr("thead input", "aria-checked").await?.as_deref() != Some("mixed") {
        bail!("select-all is not mixed with one row selected");
    }
    // Source order: Cherry, Apple, Banana, Date, Elder.
    d.click(&select_box(3)).await?;
    eventually_text(d, "#selection", "Apple,Banana", "a row's box").await?;
    if !d.text("#clicked").await?.is_empty() {
        bail!("a row's box also clicked its row");
    }

    d.click(STOCK).await?;
    eventually_text(d, "#sort", "Stock ascending", "a Stock click").await?;
    // Banana (0) first now, still selected; Cherry (3) next, not.
    eventually(d, "the selection to follow its rows", async |d| {
        Ok(d.attr("tbody tr:nth-child(1)", "aria-selected")
            .await?
            .as_deref()
            == Some("true")
            && d.attr("tbody tr:nth-child(2)", "aria-selected")
                .await?
                .as_deref()
                == Some("false"))
    })
    .await?;
    let selected = d.style("tbody tr:nth-child(1)", "background-color").await?;
    let plain = d.style("tbody tr:nth-child(2)", "background-color").await?;
    if selected == plain {
        bail!("a selected row looks like the others: {selected}");
    }

    d.click(NAME).await?;
    let expected = match d.platform() {
        Platform::Android => "Stock ascending,Name ascending",
        _ => "Name ascending",
    };
    eventually_text(d, "#sort", expected, "a Name click").await?;

    d.click(&select_box(0)).await?;
    eventually_text(
        d,
        "#selection",
        "Apple,Banana,Cherry,Date,Elder",
        "select-all",
    )
    .await?;
    if d.attr("thead input", "aria-checked").await?.is_some() {
        bail!("select-all still mixed with every row selected");
    }
    Ok(())
}

e2e::scenario!(
    rows_select_by_box_and_keep_it_through_a_sort,
    "/table/select",
    rows_select_and_sort
);

/// 1156-1c: Shift adds a column after the sorted ones and ranks both headers.
#[test]
fn a_shift_click_adds_a_sorted_column() {
    block_on(async {
        let fixture = Fixture::open("/table/select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        click_with(page, STOCK, 0).await;
        wait_text(page, "#sort", "Stock ascending").await;
        click_with(page, NAME, SHIFT).await;
        wait_text(page, "#sort", "Stock ascending,Name ascending").await;
        // Ties on Stock break by Name: Cherry before Date, Apple before Elder.
        rows(page, ":Banana:0|:Cherry:3|:Date:3|:Apple:12|:Elder:12")
            .await
            .unwrap();
        assert_eq!(count(page, "[aria-sort]").await, 2.0);
        assert_eq!(count(page, "[data-sort-order]").await, 2.0);
        let snapshot = ax::snapshot(page, "table").await.unwrap();
        assert!(snapshot.contains("sort order 2"), "{snapshot}");

        click_with(page, STOCK, SHIFT).await;
        wait_text(page, "#sort", "Stock descending,Name ascending").await;
        click_with(page, STOCK, SHIFT).await;
        wait_text(page, "#sort", "Name ascending").await;
        assert_eq!(count(page, "[data-sort-order]").await, 0.0);

        // The keyboard's way: Shift+Enter on a focused header adds it too.
        page.find_element(STOCK)
            .await
            .unwrap()
            .focus()
            .await
            .unwrap();
        keyboard::press_shift(page, keyboard::ENTER).await.unwrap();
        wait_text(page, "#sort", "Name ascending,Stock ascending").await;

        fixture.console.assert_clean("multi-sorting").unwrap();
        fixture.close().await.unwrap();
    });
}

const SHIFT: i64 = 8;

/// A mouse click at `selector`'s centre with CDP `modifiers` held.
async fn click_with(page: &Page, selector: &str, modifiers: i64) {
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
    };
    let point = page
        .find_element(selector)
        .await
        .unwrap()
        .clickable_point()
        .await
        .unwrap();
    for kind in [
        DispatchMouseEventType::MousePressed,
        DispatchMouseEventType::MouseReleased,
    ] {
        let event = DispatchMouseEventParams::builder()
            .r#type(kind)
            .x(point.x)
            .y(point.y)
            .button(MouseButton::Left)
            .click_count(1)
            .modifiers(modifiers)
            .build()
            .unwrap();
        page.execute(event).await.unwrap();
    }
}

async fn wait_text(page: &Page, selector: &str, expected: &str) {
    wait::for_js_true(
        page,
        &format!("document.querySelector({selector:?}).textContent === {expected:?}"),
        &format!("{selector} to read {expected}"),
    )
    .await
    .unwrap();
}

/// 1156-1b: page buttons turn the page, a sort goes back to page 1, and a new
/// page size keeps the first shown row on screen.
async fn the_pages_turn_sort_and_resize<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const RANGE: &str = "[data-slot=range]";
    const LIVE: &str = "[data-slot=range] + [role=status]";
    const FIRST: &str = "tbody th";
    // Empty until a change: the first range is no news.
    eventually_text(d, LIVE, "", "the first render").await?;
    let shows = async |d: &mut D, range: &str, first: &str, after: &str| -> Result<()> {
        eventually_text(d, RANGE, range, after).await?;
        eventually_text(d, FIRST, first, after).await
    };
    shows(d, "1–3 of 7", "Fig", "the first page").await?;
    d.click("button[aria-label=\"Go to next page\"]").await?;
    shows(d, "4–6 of 7", "Cherry", "Next").await?;
    eventually_text(d, "#page", "2", "onpagechange").await?;
    eventually_text(d, LIVE, "4–6 of 7", "the page turn's announcement").await?;
    d.click(SORT).await?;
    shows(d, "1–3 of 7", "Apple", "a sort").await?;
    eventually_text(d, "#page", "1", "a sort's page reset").await?;
    d.click("button[aria-label=\"Go to page 3\"]").await?;
    shows(d, "7–7 of 7", "Grape", "page 3").await?;
    d.click("[role=combobox]").await?;
    eventually(d, "the page sizes", async |d| {
        d.exists("[role=listbox]").await
    })
    .await?;
    d.click("[role=option]:nth-child(2)").await?;
    shows(d, "6–7 of 7", "Fig", "5 a page").await
}

e2e::scenario!(
    a_paged_table_turns_sorts_and_resizes,
    "/table/paged",
    the_pages_turn_sort_and_resize
);

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
    the_quick_filter_narrows_the_rows
);

/// 1156-3b/3c: a header filter narrows by value once typing settles; the menu's
/// Filter opens a dialog on the value field, whose operator and Clear apply at once.
async fn the_column_filters_narrow_the_rows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const LIVE: &str = "table + [role=status]";
    const STOCK: &str = "input[aria-label=\"Filter Stock\"]";
    const POPOVER: &str = "[data-filter-popover]";
    const MENU: &str = "[aria-label=\"Name column options\"]";
    d.click(STOCK).await?;
    d.type_text("3").await?;
    eventually_text(d, "#filters", "Stock Equals 3", "a header filter").await?;
    eventually_text(d, "tbody th", "Grape", "Stock = 3").await?;
    eventually_text(d, LIVE, "1 row", "the settled count").await?;
    d.press(keyboard::BACKSPACE).await?;
    eventually_text(d, "tbody th", "Fig", "an emptied header filter").await?;

    // Filter sits above Hide column and the Columns submenu, the last entries.
    d.focus(MENU).await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, "[role=menuitem]", "the opened column menu").await?;
    d.press(keyboard::END).await?;
    d.press(keyboard::ARROW_UP).await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(d, "[role=menuitem]:focus", "Filter", "End, Up, Up").await?;
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
    eventually_focused(d, &format!("{POPOVER} select"), "Shift+Tab").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_text(d, "#filters", "Name DoesNotContain an", "the next operator").await?;
    eventually_text(d, "tbody th", "Fig", "Name lacks an").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the closed filter", async |d| {
        Ok(!d.exists(POPOVER).await?)
    })
    .await?;
    eventually_focused(d, MENU, "Escape").await?;

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
    the_column_filters_narrow_the_rows
);

/// 1156-0b: with `row_key`, a row prepended on top leaves the others' nodes alone.
#[test]
fn a_keyed_row_keeps_its_node_when_a_row_is_prepended() {
    block_on(async {
        let fixture = Fixture::open("/table/rows", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        page.evaluate("document.querySelector('tr[data-name=\"Apple\"]').marked = true")
            .await
            .unwrap();
        page.find_element("#prepend")
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('tbody tr').dataset.name === 'Apricot' \
             && document.querySelector('tr[data-name=\"Apple\"]').marked === true",
            "Apricot on top, Apple on its own node",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("prepending a row").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("table", "/table")
        .focusable(SORT)
        .targets(SORT)
        .run();
}

#[test]
fn a_selectable_table_meets_the_baseline() {
    Suite::new("table_select", "/table/select")
        .focusable("tbody td[data-select] input")
        // The box's control, so its own hidden input is no neighbour (as on `Checkbox`).
        .targets_spaced("td[data-select] span:has(> input)")
        .targets(SORT)
        .run();
}

#[test]
fn a_wide_table_meets_the_baseline() {
    Suite::new("table_wide", "/table/wide").run();
}

#[test]
fn a_paged_table_meets_the_baseline() {
    Suite::new("table_paged", "/table/paged").run();
}

#[test]
fn a_header_click_sorts_and_flips_the_rows() {
    block_on(async {
        let fixture = Fixture::open("/table", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        rows(page, "Cherry:3 left|Apple:12 left|Banana:0 left")
            .await
            .unwrap();
        click(page).await.unwrap();
        rows(page, "Apple:12 left|Banana:0 left|Cherry:3 left")
            .await
            .unwrap();
        click(page).await.unwrap();
        rows(page, "Cherry:3 left|Banana:0 left|Apple:12 left")
            .await
            .unwrap();
        // The third click unsorts: source order, no `aria-sort`.
        click(page).await.unwrap();
        rows(page, "Cherry:3 left|Apple:12 left|Banana:0 left")
            .await
            .unwrap();
        assert_eq!(count(page, "th[aria-sort]").await, 0.0);

        fixture.console.assert_clean("sorting a table").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The sort button draws the library's ring, not the UA's `auto` outline, and
/// keeps focus while Enter re-sorts the rows under it (todo 449).
#[test]
fn the_sort_button_keeps_focus_and_draws_the_library_ring() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/table", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, SORT, 5).await.unwrap();
            let outline: String = page
                .evaluate(format!(
                    "getComputedStyle(document.querySelector({SORT:?})).outlineStyle"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(outline, "solid", "at {at}: the sort button's focus ring");

            for expected in [
                "Apple:12 left|Banana:0 left|Cherry:3 left",
                "Cherry:3 left|Banana:0 left|Apple:12 left",
            ] {
                keyboard::press(page, keyboard::ENTER).await.unwrap();
                rows(page, expected)
                    .await
                    .unwrap_or_else(|e| panic!("at {at}: {e}"));
                e2e::passes::focus::assert_focused(page, SORT, "sorting by keyboard")
                    .await
                    .unwrap_or_else(|e| panic!("at {at}: {e}"));
            }

            fixture.console.assert_clean("sorting by keyboard").unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// `aria-sort` sits on the sorted header only; an unsorted sortable header
/// still hints its arrow on hover, via `data-sortable` (todo 587).
#[test]
fn only_the_sorted_header_carries_aria_sort_and_the_others_still_hint() {
    block_on(async {
        let fixture = Fixture::open("/table/wide", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let first = "th[data-sortable]:nth-child(1)";
        let second = "th[data-sortable]:nth-child(2)";

        assert_eq!(count(page, "th[aria-sort]").await, 0.0);
        e2e::passes::pointer::hover(page, second).await.unwrap();
        arrow_opacity(page, second, "0.5").await.unwrap();

        page.find_element(&format!("{first} button"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('{first}').getAttribute('aria-sort') === 'ascending'"),
            "the first header to read ascending",
        )
        .await
        .unwrap();
        assert_eq!(count(page, "th[aria-sort]").await, 1.0);
        arrow_opacity(page, first, "1").await.unwrap();
        e2e::passes::pointer::hover(page, second).await.unwrap();
        arrow_opacity(page, second, "0.5").await.unwrap();

        fixture.console.assert_clean("hinting a sort").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A wide table scrolls in its `ScrollArea` (588), which ArrowRight scrolls from a
/// focused header button (654); the buttons make the area no stop (1156-2c).
#[test]
fn a_wide_table_scrolls_from_a_focused_header() {
    const REGION: &str = AREA;
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/table/wide", viewport).await.unwrap();
            let page = &fixture.page;

            let state: String = page
                .evaluate(format!(
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

            page.evaluate("document.querySelector('th button').focus()")
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
        }
    });
}

#[test]
fn an_empty_table_shows_its_empty_slot_across_every_column() {
    block_on(async {
        let fixture = Fixture::open("/table/empty", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "(() => { const td = document.querySelector('tbody td'); \
             return !!td && td.colSpan === 2 && td.textContent === 'No fruit'; })()",
            "the empty row to span both columns",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("an empty table").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1156-2a: `width` on the header sizes the column, padding included; the
/// rendered header sorts like a text one.
#[test]
fn a_column_width_and_a_rendered_header_reach_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/table/widths", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "(() => { const w = s => document.querySelector(s).getBoundingClientRect().width; \
             return Math.abs(w('thead th') - 200) <= 1 && Math.abs(w('tbody td') - 200) <= 1; })()",
            "the first column to be 200px wide",
        )
        .await
        .unwrap();
        assert_eq!(count(page, "th[data-sortable]").await, 1.0);
        page.find_element("thead th:nth-child(2) button")
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "(() => { const th = document.querySelector('thead th:nth-child(2)'); \
             return th.getAttribute('aria-sort') === 'ascending' \
             && th.querySelector('small').textContent === '(boxes)'; })()",
            "the rendered header to sort",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a sized table").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1156-2b: the column menu sorts, adds to the sort, hides a column and shows
/// it again from the Columns submenu.
#[test]
fn a_column_menu_sorts_hides_and_shows_columns() {
    block_on(async {
        let fixture = Fixture::open("/table/menu", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // Opens `header`'s menu, then clicks the item labelled `item` once it shows.
        async fn pick(page: &Page, header: &str, item: &str) {
            page.find_element(format!("button[aria-label=\"{header} column options\"]"))
                .await
                .unwrap()
                .click()
                .await
                .unwrap();
            choose(page, item).await;
        }
        async fn choose(page: &Page, item: &str) {
            let find = format!(
                "[...document.querySelectorAll('[role^=menuitem]')]\
                 .find(e => e.textContent.trim() === {item:?})"
            );
            wait::for_js_true(page, &format!("!!{find}"), item)
                .await
                .unwrap();
            page.evaluate(format!("{find}.click()")).await.unwrap();
        }
        let sort_of = |column: usize| {
            format!(
                "document.querySelector('thead th:nth-child({column})')?.getAttribute('aria-sort')"
            )
        };

        // Todo 1260: with a mouse, the menu button shows on its header's hover or focus.
        const STOCK_MENU: &str = "button[aria-label=\"Stock column options\"]";
        // Headless Chromium has no hover-capable pointer and CDP cannot emulate
        // one: match the fine-pointer rule's selector by hand.
        let menu_opacity = |expected: &str| {
            format!(
                "(() => {{ const all = rs => [...rs].flatMap(r => [r, ...all(r.cssRules ?? [])]); \
                 const rule = [...document.styleSheets, ...document.adoptedStyleSheets] \
                 .flatMap(s => all(s.cssRules)) \
                 .find(r => r.conditionText?.includes('pointer: fine')).cssRules[0]; \
                 const faded = document.querySelector({STOCK_MENU:?}).matches(rule.selectorText) \
                 && rule.style.opacity === '0'; return (faded ? '0' : '1') === {expected:?}; }})()"
            )
        };
        wait::for_js_true(page, &menu_opacity("0"), "the idle menu button faded")
            .await
            .unwrap();
        page.evaluate(format!("document.querySelector({STOCK_MENU:?}).focus()"))
            .await
            .unwrap();
        wait::for_js_true(page, &menu_opacity("1"), "the focused menu button shown")
            .await
            .unwrap();

        pick(page, "Stock", "Sort descending").await;
        wait::for_js_true(
            page,
            &format!(
                "{} === 'descending' && document.querySelector('tbody th').textContent === 'Apple'",
                sort_of(2)
            ),
            "Stock to sort descending",
        )
        .await
        .unwrap();

        pick(page, "Name", "Add to sort").await;
        wait::for_js_true(
            page,
            &format!(
                "{} === 'ascending' && document.querySelectorAll('[data-sort-order]').length === 2",
                sort_of(1)
            ),
            "Name to sort second",
        )
        .await
        .unwrap();

        pick(page, "Stock", "Hide column").await;
        wait::for_js_true(
            page,
            "document.querySelectorAll('thead th').length === 1 \
             && document.querySelectorAll('tbody td').length === 0",
            "Stock to hide",
        )
        .await
        .unwrap();
        // The hidden column still sorts first.
        assert_eq!(
            page.evaluate("document.querySelector('tbody th').textContent")
                .await
                .unwrap()
                .into_value::<String>()
                .unwrap(),
            "Apple"
        );

        pick(page, "Name", "Columns").await;
        choose(page, "Stock").await;
        wait::for_js_true(
            page,
            "document.querySelectorAll('thead th').length === 2 \
             && document.querySelectorAll('tbody td').length === 3",
            "Stock to show again",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("a table with column menus")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn count(page: &Page, selector: &str) -> f64 {
    page.evaluate(format!("document.querySelectorAll({selector:?}).length"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn arrow_opacity(page: &Page, header: &str, expected: &str) -> Result<()> {
    wait::for_js_true(
        page,
        &format!(
            "getComputedStyle(document.querySelector('{header} svg')).opacity === {expected:?}"
        ),
        &format!("{header}'s arrow at opacity {expected}"),
    )
    .await
}

/// Todo 742: the row header column is `th scope="row"`, exposed as a row
/// header, and looks like the cells beside it.
#[test]
fn a_row_header_column_names_its_rows() {
    block_on(async {
        let fixture = Fixture::open("/table", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        click(page).await.unwrap();
        rows(page, "Apple:12 left|Banana:0 left|Cherry:3 left")
            .await
            .unwrap();
        let same_look: bool = page
            .evaluate(
                "(() => { const ths = document.querySelectorAll('tbody th[scope=row]'); \
                 const td = document.querySelector('tbody td'); \
                 return ths.length === 3 && getComputedStyle(ths[0]).fontWeight === getComputedStyle(td).fontWeight; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(same_look, "three row headers, set like the cells");
        let snapshot = ax::snapshot(page, "table").await.unwrap();
        assert_eq!(snapshot.matches("rowheader").count(), 3, "{snapshot}");
        fixture.close().await.unwrap();
    });
}

async fn click(page: &Page) -> Result<()> {
    page.find_element(SORT).await?.click().await?;
    Ok(())
}

/// 1156-2d: scrolled to its far end, the checkbox and Name columns hold at the
/// start edge and Supplier at the end, over the scrolled cells, in LTR and RTL.
#[test]
fn pinned_columns_hold_at_their_edges_in_both_directions() {
    block_on(async {
        for (route, rtl) in [("/table/pinned", false), ("/table/pinned-rtl", true)] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            let sign = if rtl { -1 } else { 1 };
            // Retried: the area may not overflow yet on the first poll.
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const r = document.querySelector({AREA:?}); \
                     if (!r) return false; r.scrollLeft = {sign} * r.scrollWidth; \
                     return Math.abs(r.scrollLeft) > 300; }})()"
                ),
                &format!("{route} to scroll"),
            )
            .await
            .unwrap();
            // Edges as [start, end] in reading order: left/right flip in RTL.
            let state: String = page
                .evaluate(format!(
                    "(() => {{ const rtl = {rtl}; \
                     const edges = s => {{ const b = document.querySelector(s).getBoundingClientRect(); \
                       return rtl ? [b.right, b.left] : [b.left, b.right]; }}; \
                     const area = document.querySelector({AREA:?}); \
                     const a = area.getBoundingClientRect(); \
                     const start = rtl ? a.right : a.left; \
                     const end = rtl ? a.left + a.width - area.clientWidth : a.left + area.clientWidth; \
                     const select = edges('thead th[data-select]'); \
                     const name = edges('th[aria-label=Name]'); \
                     const supplier = edges('th[aria-label=Supplier]'); \
                     const origin = edges('th[aria-label=Origin]'); \
                     const near = (x, y) => Math.abs(x - y) <= 1; \
                     const cell = document.querySelector('tbody tr:nth-child(2) td[data-pin=start]'); \
                     const c = cell.getBoundingClientRect(); \
                     const hit = document.elementFromPoint(c.left + c.width / 2, c.top + c.height / 2); \
                     const bg = getComputedStyle(cell).backgroundColor; \
                     return [near(select[0], start), near(name[0], select[1]), \
                       near(supplier[1], end), rtl ? origin[0] > name[0] : origin[0] < name[0], \
                       cell.contains(hit), bg === getComputedStyle(cell.parentElement).backgroundColor, \
                       bg !== 'rgba(0, 0, 0, 0)', \
                       getComputedStyle(document.querySelector('th[aria-label=Name]')).zIndex === '3' \
                       && getComputedStyle(document.querySelector('th[aria-label=Origin]')).zIndex === '2' \
                       && getComputedStyle(cell).zIndex === '1'].join('|') + ' ' + JSON.stringify([start, end, select, name, supplier]); }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            // The edges follow, for a failure message.
            assert!(
                state.starts_with("true|true|true|true|true|true|true|true "),
                "{route}: checkbox, Name at the start edge, Supplier at the end, Origin \
                 scrolled under them; the striped pinned cell on top and opaque; a pinned \
                 header (a corner under max_height) over the others: {state}"
            );
            fixture.console.assert_clean("pinned columns").unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// 1156-2d: "Pin to start" in a column's menu pins it after the pinned ones.
async fn a_menu_pin_moves_the_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("button[aria-label=\"Origin column options\"]")
        .await?;
    eventually(d, "the column menu to open", async |d| {
        d.exists("[role=menuitem]").await
    })
    .await?;
    if d.text("[role=menuitem]").await? != "Pin to start" {
        bail!("the first entry is not Pin to start");
    }
    d.click("[role=menuitem]").await?;
    eventually_text(d, "#pinned", "Name,Origin|Supplier", "Pin to start").await?;
    eventually(d, "Origin to pin past Name", async |d| {
        Ok(d.attr("th[aria-label=Origin]", "data-pin")
            .await?
            .as_deref()
            == Some("start")
            && d.rect("th[aria-label=Origin]").await?.x > d.rect("th[aria-label=Name]").await?.x)
    })
    .await
}

e2e::scenario!(
    a_column_menu_pins_a_column,
    "/table/pinned",
    a_menu_pin_moves_the_column
);

/// The body reads `name:cell` per row, in order, joined by `|`.
async fn rows(page: &Page, expected: &str) -> Result<()> {
    wait::for_js_true(
        page,
        &format!(
            "(() => [...document.querySelectorAll('tbody tr')]\
             .map(tr => [...tr.cells].map(td => td.textContent).join(':'))\
             .join('|') === {expected:?})()"
        ),
        &format!("the rows to read {expected}"),
    )
    .await
}
