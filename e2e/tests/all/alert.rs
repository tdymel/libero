//! `Alert`: contrast in every severity and variant, the close button, and a
//! title that wraps rather than being cut.

use e2e::browser::block_on;
use e2e::passes::{contrast, keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const CLOSE: &str = "#dismissible [data-slot=close]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("alert", "/alert")
        .waive(contrast::TODO_297)
        .focusable(CLOSE)
        // `sm`: drawn 20x20, pressed in a 24x24 box.
        .targets(CLOSE)
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
        assert!(
            scroll <= client + 1.0,
            "the title is cut: {scroll} of {client}px shown"
        );
        assert!(
            lines > 1.5,
            "the long title should wrap, it takes {lines} lines"
        );

        fixture.console.assert_clean("the long title").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `parts` reaches the inner parts, the instance `sx` wins a tie, a nested
/// `Alert` is not reached, and a changed part restyles.
#[test]
fn parts_style_the_inner_parts() {
    block_on(async {
        let fixture = Fixture::open("/alert", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#styled-parts").await.unwrap();
        let style = |selector: &str, property: &str| {
            format!("getComputedStyle(document.querySelector('{selector}')).{property}")
        };
        let read = |js: String| async move {
            page.evaluate(js)
                .await
                .unwrap()
                .into_value::<String>()
                .unwrap()
        };

        let title = "#styled-parts > [data-slot=body] > [data-slot=title]";
        let message = "#styled-parts > [data-slot=body] > [data-slot=message]";
        assert_eq!(read(style(title, "fontStyle")).await, "italic");
        assert_eq!(read(style(title, "letterSpacing")).await, "4px");
        assert_eq!(read(style(message, "paddingLeft")).await, "8px");
        assert_eq!(
            read(style("#nested [data-slot=title]", "fontStyle")).await,
            "normal",
            "a nested Alert took the outer one's parts"
        );

        pointer::click(page, "#toggle-parts").await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '4px'", style(message, "paddingLeft")),
            "the changed part restyles the message",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("styled parts").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1647. On a fill or tint a link is the text colour, so it keeps its
/// underline at rest; on the page's surface it keeps the hover-only default.
#[test]
fn a_link_on_a_fill_or_tint_is_underlined_at_rest() {
    block_on(async {
        let fixture = Fixture::open("/alert", Viewport::Desktop).await.unwrap();
        wait::for_js_true(
            &fixture.page,
            "(() => { const links = [...document.querySelectorAll('[id$=-filled] a, [id$=-tonal] a, [id$=-outlined] a')]; \
             return links.length === 12 && links.every(a => \
                 (getComputedStyle(a).textDecorationLine === 'underline') === !a.closest('[id$=-outlined]')); })()",
            "filled and tonal links underlined, outlined ones not",
        )
        .await
        .unwrap();
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
