//! `Header`: the banner landmark, and the page-side fix that keeps focus from
//! hiding under it (WCAG 2.4.11).

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, TAB};
use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("header", "/header").focusable("#home").run();
}

/// Scrolls `#item-5` to just under the stuck banner's bottom edge with `#item-6`
/// focused, presses Shift+Tab, and returns how far the focused button's bottom
/// sits below the banner's (negative: entirely covered).
async fn clearance_after_shift_tab(padded: bool) -> f64 {
    let fixture = Fixture::open("/header", Viewport::Desktop).await.unwrap();
    let page = &fixture.page;
    wait::for_visible(page, "#banner").await.unwrap();
    page.evaluate(
        "(() => { document.querySelector('#item-6').focus(); \
         const covered = document.querySelector('#item-5').getBoundingClientRect().bottom; \
         const banner = document.querySelector('#banner').getBoundingClientRect().height; \
         window.scrollBy(0, covered - banner + 2); })()",
    )
    .await
    .unwrap();
    if padded {
        page.evaluate(
            "document.documentElement.style.scrollPaddingTop = 'var(--lsx-header-height-md)'",
        )
        .await
        .unwrap();
    }
    keyboard::press_shift(page, TAB).await.unwrap();

    let clearance: f64 = page
        .evaluate(
            "(() => { if (document.activeElement.id !== 'item-5') return NaN; \
             return document.activeElement.getBoundingClientRect().bottom \
             - document.querySelector('#banner').getBoundingClientRect().bottom; })()",
        )
        .await
        .unwrap()
        .into_value()
        .unwrap();
    fixture.console.assert_clean("the header fixture").unwrap();
    fixture.close().await.unwrap();
    clearance
}

/// Without `scroll-padding-top` the browser leaves a button it thinks is on
/// screen where it is, under a sticky header; the docs tell callers to set it.
#[test]
fn scroll_padding_keeps_focus_clear_of_a_sticky_header() {
    block_on(async {
        let bare = clearance_after_shift_tab(false).await;
        assert!(bare < 0.0, "the fixture no longer hides the button: {bare}");
        let padded = clearance_after_shift_tab(true).await;
        assert!(padded > 0.0, "focus still under the header: {padded}");
    });
}
