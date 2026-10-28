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
    typing_at("/keyboardless", None).await
}

/// As [`typing`] on `route`, after a tap on `opener` when there is one.
async fn typing_at(route: &str, opener: Option<&str>) -> Fixture {
    let fixture = Fixture::open(route, Viewport::Mobile).await.unwrap();
    let page = &fixture.page;
    page.execute_bounded(SetDeviceMetricsOverrideParams::new(360, 780, 1.0, true))
        .await
        .unwrap();
    if let Some(opener) = opener {
        tap(page, opener).await.unwrap();
        let shown = format!("!!document.querySelector({FIELD:?})");
        wait::for_js_true(page, &shown, "the field in the modal")
            .await
            .unwrap();
    }
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

/// Todo 4777: the field last in a modal, Tab past the text box stays in the modal.
#[test]
fn tab_past_the_text_box_stays_in_a_modal() {
    block_on(async {
        let fixture = typing_at("/keyboardless/modal", Some("#open")).await;
        let page = &fixture.page;
        keyboard::press(page, keyboard::TAB).await.unwrap();
        let kept = "(() => { const at = document.activeElement; \
             return !!at && !at.matches('[data-slot=text-input]') \
               && !!at.closest('#modal'); })()";
        let outside = "document.activeElement?.id === 'behind'";
        wait::for_js_true(page, &format!("{kept} || {outside}"), "focus moved")
            .await
            .unwrap();
        let at: String = page
            .evaluate("document.activeElement?.id || document.activeElement?.tagName")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            page.evaluate(kept)
                .await
                .unwrap()
                .into_value::<bool>()
                .unwrap(),
            "focus left the modal, on {at}"
        );
        fixture.console.assert_clean("Tab in a modal").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 4607, 4778: a refused text box draws an error border and is described by a line
/// shown in the sheet.
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
               && !!line.closest({SHEET:?}) && line.getBoundingClientRect().height > 0 \
               && getComputedStyle(own).borderTopColor !== {calm:?}; }})()"
        );
        wait::for_js_true(
            page,
            &refused,
            "aria-invalid, described by a line in the sheet and an error border",
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
