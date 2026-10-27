//! A picker field's sheet under a finger: Tab out of it (todo 4606), the refused text box (4607)
//! and its name (4608).

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::passes::pointer::tap;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

const SHEET: &str = "[role=dialog]";
const TYPED: &str = "[role=dialog] [data-slot=text-input]";
const FIELD: &str = "#due input[data-controlled]";

/// The sheet open, its text box focused by a tap.
async fn typing() -> Fixture {
    let fixture = Fixture::open("/keyboardless", Viewport::Mobile)
        .await
        .unwrap();
    let page = &fixture.page;
    page.execute_bounded(SetDeviceMetricsOverrideParams::new(360, 780, 1.0, true))
        .await
        .unwrap();
    tap(page, FIELD).await.unwrap();
    let shown = format!("!!document.querySelector({TYPED:?})");
    wait::for_js_true(page, &shown, "the sheet's text box")
        .await
        .unwrap();
    tap(page, TYPED).await.unwrap();
    let focused = format!("document.querySelector({TYPED:?}) === document.activeElement");
    wait::for_js_true(page, &focused, "the text box focused by the tap")
        .await
        .unwrap();
    fixture
}

async fn border(page: &Page) -> String {
    page.evaluate(format!(
        "getComputedStyle(document.querySelector({TYPED:?})).borderTopColor"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// Todo 4606: Tab past the text box goes on to the page's next stop, not back into the sheet.
#[test]
fn tab_past_the_text_box_leaves_the_sheet() {
    block_on(async {
        let fixture = typing().await;
        let page = &fixture.page;
        keyboard::press(page, keyboard::TAB).await.unwrap();
        let next = format!(
            "!document.querySelector({SHEET:?}) \
             && document.activeElement === document.getElementById('after')"
        );
        wait::for_js_true(page, &next, "the button after the field, the sheet closed")
            .await
            .unwrap();
        fixture.console.assert_clean("Tab out of a sheet").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4607: a refused text box draws an error border and is described by the error line.
#[test]
fn a_refused_text_box_shows_its_error() {
    block_on(async {
        let fixture = typing().await;
        let page = &fixture.page;
        let calm = border(page).await;
        page.evaluate(format!("document.querySelector({TYPED:?}).select()"))
            .await
            .unwrap();
        keyboard::insert_text(page, "foo").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        let refused = format!(
            "(() => {{ const own = document.querySelector({TYPED:?}); \
             if (!own || own.getAttribute('aria-invalid') !== 'true') return false; \
             const line = document.getElementById(own.getAttribute('aria-describedby')); \
             return !!line && line.textContent.trim().length > 0 \
               && getComputedStyle(own).borderTopColor !== {calm:?}; }})()"
        );
        wait::for_js_true(
            page,
            &refused,
            "aria-invalid, described and an error border",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4608: the text box is named by the field's label and a hint, not the label alone.
#[test]
fn the_text_box_is_told_apart_from_the_field() {
    block_on(async {
        let fixture = typing().await;
        let page = &fixture.page;
        let named = format!(
            "(() => {{ const ids = document.querySelector({TYPED:?}).getAttribute('aria-labelledby'); \
             return ids.split(' ').map((id) => document.getElementById(id)?.textContent).join(' ') \
               === 'Due Type it in'; }})()"
        );
        wait::for_js_true(page, &named, "named 'Due Type it in'")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}
