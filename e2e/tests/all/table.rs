//! `Table`: a header click sorts, a second flips it, a third unsorts, and a custom cell body
//! moves with its row. Plain cells draw their text inline (todo 29).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text, linger};
use e2e::passes::{focus, keyboard};
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
    d.focus("[role=region]").await?;
    for _ in 0..8 {
        if d.is_focused(LAST).await? {
            break;
        }
        d.press(keyboard::TAB).await?;
    }
    eventually_focused(d, LAST, "Tab").await?;
    eventually(d, "the region to show the last header", async |d| {
        let region = d.rect("[role=region]").await?;
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

/// A wide table scrolls in its region, a named tab stop with the library ring (588), which
/// ArrowRight scrolls once focused (654).
#[test]
fn a_wide_table_scrolls_in_a_named_focusable_region() {
    const REGION: &str = "[role=region]";
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
                     r.getAttribute('aria-labelledby') === c.id, \
                     c.textContent].join('|'); }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(
                state, "true|true|true|Fruit catalogue",
                "at {at}: overflows itself, not the page; named by its caption"
            );

            let ring = focus::assert_focus_ring(page, REGION, 3)
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            focus::assert_ring_contrast(&ring).unwrap_or_else(|e| panic!("at {at}: {e}"));

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
