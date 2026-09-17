//! `Table`: a header click sorts, a second flips it, and a custom cell body
//! moves with its row. Plain cells draw their text inline (todo 29).

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{focus, keyboard};
use e2e::{Fixture, Viewport, ax, wait};

const SORT: &str = "th[data-sortable] button";

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

/// A wide table scrolls inside its region, not the page; the region is a
/// named tab stop with the library ring (todo 588), which ArrowRight scrolls
/// once it holds focus (654).
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
