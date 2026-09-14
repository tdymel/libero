//! `Alert`: contrast in every severity and variant, the close button, and a
//! title that wraps rather than being cut.

use e2e::browser::block_on;
use e2e::passes::{contrast, keyboard};
use e2e::{Fixture, Suite, Viewport, wait};

const CLOSE: &str = "#dismissible [data-slot=close]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("alert", "/alert")
        .waive(contrast::TODO_297)
        .focusable(CLOSE)
        // `sm`: 20x20, clear of everything else in the alert.
        .targets_spaced(CLOSE)
        .run();
}

/// An ellipsis at 320px cut the title for every sighted reader, with no way to
/// read the rest (WCAG 1.4.10).
#[test]
fn a_long_title_wraps_rather_than_being_cut() {
    block_on(async {
        let fixture = Fixture::open("/alert", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#long-title").await.unwrap();

        let (scroll, client, lines): (f64, f64, f64) = page
            .evaluate(
                "(() => { const t = document.querySelector('#long-title [data-slot=title]'); \
                 const s = getComputedStyle(t); \
                 return [t.scrollWidth, t.clientWidth, \
                         t.getBoundingClientRect().height / parseFloat(s.lineHeight)]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(scroll <= client + 1.0, "the title is cut: {scroll} of {client}px shown");
        assert!(lines > 1.5, "the long title should wrap, it takes {lines} lines");

        fixture.console.assert_clean("the long title").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Enter on the close button calls `onclose`. Where focus goes next is the
/// caller's: the alert is gone, so it falls to `<body>`.
#[test]
fn the_close_button_works_from_the_keyboard() {
    block_on(async {
        let fixture = Fixture::open("/alert", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, CLOSE, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#dismissible')",
            "Enter to close the alert",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("closing an alert").unwrap();
        fixture.close().await.unwrap();
    });
}
