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
    if !padded {
        page.evaluate("document.documentElement.style.scrollPaddingTop = '0px'")
            .await
            .unwrap();
    }
    // The banner publishes both on `:root` itself.
    let published: String = page
        .evaluate(
            "(() => { const root = getComputedStyle(document.documentElement); \
             return root.getPropertyValue('--lsx-header-height').trim() + '|' + root.scrollPaddingTop; })()",
        )
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(
        published.starts_with("64px|"),
        "the banner published no height: {published}"
    );
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

/// The opted-in banner's size is what `:root` gets; a static header and a
/// sticky one without `publish_height` never publish.
#[test]
fn only_an_opted_in_header_publishes_its_height() {
    block_on(async {
        let fixture = Fixture::open("/header-static", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#nested").await.unwrap();
        let heights: String = page
            .evaluate(
                "(() => { const root = getComputedStyle(document.documentElement); \
                 const lg = root.getPropertyValue('--lsx-header-height-lg').trim(); \
                 return [root.getPropertyValue('--lsx-header-height').trim(), lg, \
                 document.querySelector('#nested').getBoundingClientRect().height].join('|'); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let parts: Vec<&str> = heights.split('|').collect();
        assert_eq!(
            parts[0], parts[1],
            "root is not the banner's lg height: {heights}"
        );
        assert_eq!(
            parts[2], "48",
            "the nested header lost its own xs size: {heights}"
        );
        fixture
            .console
            .assert_clean("the static header fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Without `scroll-padding-top` the browser leaves a button it thinks is on
/// screen where it is, under a sticky header; the banner sets it on `:root`.
#[test]
fn scroll_padding_keeps_focus_clear_of_a_sticky_header() {
    block_on(async {
        let bare = clearance_after_shift_tab(false).await;
        assert!(bare < 0.0, "the fixture no longer hides the button: {bare}");
        let padded = clearance_after_shift_tab(true).await;
        assert!(padded > 0.0, "focus still under the header: {padded}");
    });
}
