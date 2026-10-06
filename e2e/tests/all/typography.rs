//! `Text`, `Title`, `Blockquote`, `Mark` and inline `Code`: contrast in both
//! schemes, and text that reflows in a 320px column.

use e2e::browser::block_on;
use e2e::{Fixture, Suite, Viewport, ax, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("typography", "/typography")
        .contrast_covers("#marks")
        .contrast_covers("#code-rust")
        .contrast_covers("#quote-default figcaption")
        .contrast_covers("#text-color")
        .contrast_covers("#kbds")
        .run();
}

/// Every tint, the dimmed attribution under it, and a link in an attribution.
#[test]
fn every_quote_colour_meets_the_baseline() {
    Suite::new("typography_quotes", "/typography/quotes")
        .contrast_covers("#quotes")
        .contrast_covers("#attribution-link")
        .focusable("#attribution-link")
        .run();
}

/// How far `selector` pokes out past the right edge of its paragraph (a heading: its column), in px.
async fn overflow(page: &chromiumoxide::Page, selector: &str) -> f64 {
    page.evaluate(format!(
        "(() => {{ const el = document.querySelector({selector:?}); \
         const column = (el.closest('p') ?? el.parentElement).getBoundingClientRect(); \
         return Math.max(...[...el.getClientRects()].map(r => r.right)) - column.right; }})()"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// A long identifier broke out of its paragraph at 320px (WCAG 1.4.10).
#[test]
fn a_long_inline_code_wraps_inside_its_paragraph() {
    block_on(async {
        let fixture = Fixture::open("/typography", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#code-long").await.unwrap();

        let past = overflow(page, "#code-long").await;
        assert!(
            past <= 1.0,
            "the inline code runs {past}px past its paragraph"
        );

        fixture
            .console
            .assert_clean("the long inline code")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2543 and 2547: a long heading word and the largest key at 320px (WCAG 1.4.10),
/// and every key's text read out in the sentence.
#[test]
fn a_long_title_and_the_keys_stay_inside_the_column() {
    block_on(async {
        let fixture = Fixture::open("/typography", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#title-long").await.unwrap();

        for selector in ["#title-long", "#kbds kbd:last-of-type"] {
            let past = overflow(page, selector).await;
            assert!(past <= 1.0, "{selector} runs {past}px past its column");
        }

        let tree = ax::snapshot(page, "#kbds").await.unwrap();
        assert_eq!(tree.matches("Ctrl").count(), 6, "the keys' text:\n{tree}");

        fixture.console.assert_clean("the long title").unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.12's user stylesheet: nothing may be cut or overlap under it.
const TEXT_SPACING: &str = "(() => { const s = document.createElement('style'); \
    s.textContent = '* { line-height: 1.5 !important; letter-spacing: 0.12em !important; \
    word-spacing: 0.16em !important; } p { margin-bottom: 2em !important; }'; \
    document.head.appendChild(s); return true; })()";

#[test]
fn text_spacing_overrides_lose_nothing() {
    block_on(async {
        let fixture = Fixture::open("/typography", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#code-long").await.unwrap();
        page.evaluate(TEXT_SPACING).await.unwrap();

        // Every element that clips its overflow must still fit its content.
        let clipped: Vec<String> = page
            .evaluate(
                "[...document.querySelectorAll('[data-fixture-ready] *')] \
                 .filter(el => { const s = getComputedStyle(el); \
                   return (s.overflowX !== 'visible' || s.overflowY !== 'visible') \
                     && (el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1); }) \
                 .map(el => el.tagName + '#' + el.id)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            clipped.is_empty(),
            "clipped under text spacing: {clipped:?}"
        );

        for selector in [
            "#code-long",
            "#code-rust",
            "#marks mark",
            "#title-long",
            "#kbds kbd",
        ] {
            let past = overflow(page, selector).await;
            assert!(past <= 1.0, "{selector} runs {past}px past its paragraph");
        }

        fixture.console.assert_clean("text spacing").unwrap();
        fixture.close().await.unwrap();
    });
}
