//! `Dialog` outside a modal, and the plain surfaces beside it: `Paper` and
//! `VisuallyHidden`.

use e2e::browser::block_on;
use e2e::passes::{focus, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const CLOSE: &str = "#inline button[aria-label='Close']";

#[test]
fn it_meets_the_baseline() {
    Suite::new("dialog", "/dialog")
        .focusable(CLOSE)
        .focusable("#card")
        .targets(CLOSE)
        .run();
}

/// Outside a modal: a named `role=dialog` that claims no modality and takes
/// no tab stop of its own.
#[test]
fn an_inline_dialog_is_named_by_its_title_and_not_modal() {
    block_on(async {
        let fixture = Fixture::open("/dialog", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#inline").await.unwrap();

        let facts: Vec<Option<String>> = page
            .evaluate(
                "(() => { const d = document.querySelector('#inline'); \
                 const h = document.getElementById(d.getAttribute('aria-labelledby')); \
                 return [d.getAttribute('role'), d.getAttribute('aria-modal'), \
                 d.getAttribute('tabindex'), h && h.tagName + ':' + h.textContent]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            facts,
            [Some("dialog".into()), None, None, Some("H2:Filters".into())]
        );
        fixture.console.assert_clean("the inline dialog").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2947: a long word in the title breaks inside the dialog on a phone.
#[test]
fn a_long_dialog_title_stays_inside_the_dialog() {
    block_on(async {
        let fixture = Fixture::open("/dialog/long-title", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#long").await.unwrap();

        let past: f64 = page
            .evaluate(
                "(() => { const d = document.querySelector('#long'); \
                 const t = document.getElementById(d.getAttribute('aria-labelledby')); \
                 return t.getBoundingClientRect().width - d.getBoundingClientRect().width; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(past <= 1.0, "the title runs {past}px past the dialog");
        fixture
            .console
            .assert_clean("the long dialog title")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2561: `width: 100%` plus a margin stuck out of the container by the margin.
#[test]
fn an_inline_dialog_fits_its_container() {
    block_on(async {
        let fixture = Fixture::open("/dialog", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#inline").await.unwrap();

        let (overhang, page_overflow): (f64, f64) = page
            .evaluate(
                "(() => { const d = document.querySelector('#inline'); \
                 const p = d.parentElement.getBoundingClientRect(); \
                 const e = document.documentElement; \
                 return [d.getBoundingClientRect().right - p.right, e.scrollWidth - e.clientWidth]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(overhang <= 0.5, "the dialog sticks out by {overhang}px");
        assert!(
            page_overflow <= 0.0,
            "the page scrolls sideways by {page_overflow}px"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 611: outside a modal the close button calls `onclose`.
#[test]
fn the_close_button_of_an_inline_dialog_calls_onclose() {
    block_on(async {
        let fixture = Fixture::open("/dialog", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#inline").await.unwrap();
        pointer::click(page, CLOSE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#inline')",
            "the dialog to close",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("closing the inline dialog")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `component: "a"` makes a card link; it has to show the keyboard ring.
#[test]
fn a_paper_link_shows_a_focus_ring() {
    block_on(async {
        let fixture = Fixture::open("/dialog", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let ring = focus::assert_focus_ring(page, "#card", 6).await.unwrap();
        focus::assert_ring_contrast(&ring).unwrap();
        fixture.close().await.unwrap();
    });
}
