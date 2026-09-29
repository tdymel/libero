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

/// A `NativeSelect`: natively a listbox combobox, no `<select>`.
const PICKER: &str = ":is(select, [role=combobox])";

/// Steps the focused `NativeSelect` `steps` options on. Natively the first
/// arrow opens its listbox on the current option and Enter picks.
async fn pick_later<D: Driver>(d: &mut D, steps: usize) -> Result<()> {
    let native = d.platform() == Platform::Native;
    if native {
        d.press(keyboard::ARROW_DOWN).await?;
    }
    for _ in 0..steps {
        d.press(keyboard::ARROW_DOWN).await?;
    }
    if native {
        d.press(keyboard::ENTER).await?;
    }
    Ok(())
}

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
    if d.platform() == Platform::Native {
        // A click on the drawn box focuses nothing on Blitz and Tab fires no `focusin`:
        // the table finds the focused row after Tab's silent move.
        d.focus("tr[aria-rowindex=\"10000\"] input").await?;
        d.press(keyboard::TAB).await?;
        eventually_focused(d, &last_box, "Tab").await?;
        d.press(keyboard::HOME).await?;
        eventually(d, "the first rows to come back", async |d| {
            Ok(d.exists(FIRST).await? && d.is_focused(&last_box).await?)
        })
        .await?;
        // Tab wraps round to the select-all box: the last row goes with the rest.
        d.press(keyboard::TAB).await?;
        return eventually(d, "the last row to unmount once focus left it", async |d| {
            Ok(d.exists(FIRST).await? && !d.exists(LAST).await?)
        })
        .await;
    }
    // The focused row stays rendered out of view, focus still in it.
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
    the_window_follows_the_scroll
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
    linger(d, 400).await;
    row_count_is(d, "151").await
}

async fn row_count_is<D: Driver>(d: &mut D, rows: &str) -> Result<()> {
    eventually(d, &format!("aria-rowcount {rows}"), async |d| {
        Ok(d.attr("table", "aria-rowcount").await?.as_deref() == Some(rows))
    })
    .await
}

e2e::scenario!(
    end_at_the_bottom_loads_more_rows,
    "/table/infinite",
    end_loads_the_next_rows
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
    d.click(DUE).await?;
    match native_input {
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
    if !native_input {
        // Into the value field's calendar and back: the popover stays open throughout.
        let value = format!("{POPOVER} input[data-filter-value]");
        eventually_focused(d, &value, "the opened filter").await?;
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
    android: skip("Chromium's typed date segments only")
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
        // The px of the area under the header that no rendered row covers.
        const GAP: &str = "(() => { const area = document.querySelector('[data-table-scroll]'); \
             const a = area.getBoundingClientRect(); \
             const top = document.querySelector('thead').getBoundingClientRect().bottom; \
             const bottom = a.top + area.clientHeight; \
             const rows = [...document.querySelectorAll('tbody tr')].map(r => r.getBoundingClientRect()); \
             const first = Math.min(...rows.map(r => r.top)), last = Math.max(...rows.map(r => r.bottom)); \
             return Math.max(0, first - top) + Math.max(0, bottom - last); })()";
        // Firefox anchored on the drawn scrollbar layer and scrolled on after it, to the bottom.
        let anchor: String = page
            .evaluate(
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
            .evaluate(
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
            page.evaluate(format!("document.querySelector({AREA:?}).scrollTop = {at}"))
                .await
                .unwrap();
            let settled =
                wait::for_js_true(page, &format!("{GAP} < 1"), "the rows to cover the area").await;
            let (gap, top): (f64, f64) = page
                .evaluate(format!(
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
        page.evaluate(format!("{ROW}.marked = true")).await.unwrap();
        page.find_element("th[data-sortable] [data-sort-button]")
            .await
            .unwrap()
            .click()
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
                    .evaluate(FOCUSED_ROW)
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
                        .evaluate(FOCUSED_ROW)
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
            .evaluate(
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

/// 1156-5a: windowed and scrolled both ways, the header sticks and the pinned
/// columns hold at their edges, body cells in line with their headers.
#[test]
fn a_windowed_table_keeps_its_header_and_pins_while_scrolled() {
    block_on(async {
        let fixture = Fixture::open("/table/windowed-pinned", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const r = document.querySelector({AREA:?}); \
                 if (!r) return false; r.scrollLeft = r.scrollWidth; r.scrollTop = 200000; \
                 return r.scrollLeft > 300 && !!document.querySelector('tr[aria-rowindex=\"5005\"]') \
                   && !document.querySelector('tr[aria-rowindex=\"2\"]'); }})()"
            ),
            "the window to move down and right",
        )
        .await
        .unwrap();
        let state: String = page
            .evaluate(format!(
                "(() => {{ const box = s => document.querySelector(s).getBoundingClientRect(); \
                 const area = document.querySelector({AREA:?}); \
                 const a = area.getBoundingClientRect(); \
                 const end = a.left + area.clientWidth; \
                 const row = 'tr[aria-rowindex=\"5005\"] '; \
                 const select = box('thead th[data-select]'), name = box('thead th[data-pin=start]'); \
                 const origin = box('thead th:nth-child(4)'), supplier = box('thead th[data-pin=end]'); \
                 const cell = box(row + 'td[data-pin=start]'), last = box(row + 'td[data-pin=end]'); \
                 const near = (x, y) => Math.abs(x - y) <= 1; \
                 return [near(select.left, a.left), near(name.left, select.right), \
                   near(cell.left, name.left), near(cell.width, name.width), \
                   near(supplier.right, end), near(last.right, end), origin.left < name.left, \
                   near(name.top, a.top)].join('|') + ' ' + JSON.stringify([a, select, name, cell, supplier, last]); }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            state.starts_with("true|true|true|true|true|true|true|true "),
            "checkbox and Name at the start, Supplier at the end, body cells under their \
             headers, Origin scrolled under, the header at the top: {state}"
        );
        fixture
            .console
            .assert_clean("a windowed pinned table")
            .unwrap();
        fixture.close().await.unwrap();
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
    // After Filter and the moves.
    let mut pin = None;
    for index in 0..8 {
        let item = format!("[role=menuitem][data-menu-index=\"{index}\"]");
        if d.exists(&item).await? && d.text(&item).await? == "Pin to start" {
            pin = Some(item);
            break;
        }
    }
    let Some(pin) = pin else {
        bail!("no Pin to start entry");
    };
    d.click(&pin).await?;
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

/// WCAG 2.4.11: Shift+Tab up a capped table's rows never leaves the focused
/// checkbox under the sticky header.
async fn a_focused_row_stays_below_the_header<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("tbody tr:nth-child(1) input").await?;
    for _ in 0..30 {
        d.press(keyboard::TAB).await?;
        linger(d, 4).await;
    }
    // Polls each step instead of a fixed `linger(d, 100)`, 2.5 s per Tab and
    // 150 s of the table unit (823). The page may scroll a frame after the focus.
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

    d.click("[aria-label=\"Stock column options\"]").await?;
    eventually(d, "the column menu", async |d| {
        d.exists("[role=menuitem]").await
    })
    .await?;
    let mut picked = false;
    for index in 0..12 {
        let item = format!("[role=menuitem][data-menu-index=\"{index}\"]");
        if d.exists(&item).await? && d.text(&item).await?.trim() == "Filter" {
            d.click(&item).await?;
            picked = true;
            break;
        }
    }
    if !picked {
        bail!("no Filter entry");
    }
    eventually(d, "the panel from the menu", async |d| {
        d.exists(PANEL).await
    })
    .await?;
    eventually_focused(d, &format!("{LINE_1} {PICKER}"), "the menu's Filter").await
}

e2e::scenario!(
    the_filter_panel_edits_every_filter,
    "/table/filter-panel",
    the_filter_panel_edits_every_filter,
    android: skip("958: element identity on the WebView")
);
