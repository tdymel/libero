//! `Table`: a header click sorts, a second flips it, and a custom cell body
//! moves with its row. Plain cells draw their text inline (todo 29).

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const SORT: &str = "th[aria-sort] button";

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
