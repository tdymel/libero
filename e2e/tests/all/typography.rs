//! `Text`, `Title`, `Blockquote`, `Mark` and inline `Code`: contrast in both
//! schemes, and text that reflows in a 320px column.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::Driver;
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, ax, wait};

/// Todo 3966: `truncate` keeps the text on one line, `line_clamp` cuts it after its lines and
/// wins over `truncate`, `0` is off, and `start` reads right to left.
async fn truncate_cuts_to_lines<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for (one, plain, one_line, clamped) in [
        ("#text-one", "#text-plain", "#text-end", "#text-clamp"),
        ("#title-one", "#title-plain", "#title-end", "#title-clamp"),
    ] {
        let line = d.rect(one).await?.height;
        let whole = d.rect(plain).await?.height;
        ensure!(
            whole > line * 2.5,
            "{plain} fits in {whole}, a line is {line}"
        );
        let end = d.rect(one_line).await?.height;
        ensure!(
            (end - line).abs() <= 1.0,
            "{one_line} is {end} tall, a line is {line}"
        );
        let clamp = d.rect(clamped).await?.height;
        ensure!(
            (clamp - 2.0 * line).abs() <= 2.0,
            "{clamped} is {clamp} tall, two lines are {}",
            2.0 * line
        );
    }
    let line = d.rect("#text-one").await?.height;
    let start = d.rect("#text-start").await?.height;
    ensure!(
        (start - line).abs() <= 1.0,
        "#text-start is {start} tall, a line is {line}"
    );
    let direction = d.style("#text-start", "direction").await?;
    ensure!(direction == "rtl", "a start cut reads {direction}");
    let wins = d.rect("#text-clamp-wins").await?.height;
    ensure!(
        (wins - 2.0 * line).abs() <= 2.0,
        "line_clamp lost to truncate: {wins}"
    );
    let off = d.rect("#text-clamp-off").await?.height;
    let whole = d.rect("#text-plain").await?.height;
    ensure!(
        (off - whole).abs() <= 1.0,
        "line_clamp 0 cut the text: {off} of {whole}"
    );
    Ok(())
}

e2e::scenario!(
    truncate_and_line_clamp_cut_text_and_titles,
    "/typography/truncate",
    truncate_cuts_to_lines
);

/// Todos 2542, 2590: a key's size is `rem`, so at a 32px root it doubles with the
/// sentence; in `px` it stayed put.
#[test]
fn the_keys_grow_with_the_root_font_size() {
    block_on(async {
        let fixture = Fixture::open("/typography", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#kbds kbd").await.unwrap();
        const SIZES: &str = "[...document.querySelectorAll('#kbds kbd')] \
                             .map((k) => getComputedStyle(k).fontSize).join(' ')";
        let before: String = page
            .evaluate_bounded(SIZES)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            before, "10px 12px 14px 16px 20px 24px",
            "the default look moved"
        );

        page.evaluate_bounded("document.documentElement.style.fontSize = '32px'")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{SIZES} === '20px 24px 28px 32px 40px 48px'"),
            "the keys to double at a 32px root",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

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

/// Every tint, the dimmed attribution under it, and a link in the quote and in an attribution.
#[test]
fn every_quote_colour_meets_the_baseline() {
    Suite::new("typography_quotes", "/typography/quotes")
        .contrast_covers("#quotes")
        .contrast_covers("#attribution-link")
        .contrast_covers("#quote-link")
        .focusable("#attribution-link")
        .focusable("#quote-link")
        .run();
}

/// Todo 2914: a link beside text in the caption is underlined at rest, as in the quote (1.4.1).
#[test]
fn a_link_in_the_caption_is_underlined_at_rest() {
    block_on(async {
        let fixture = Fixture::open("/typography/quotes", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#attribution-link").await.unwrap();

        let lines: Vec<String> = page
            .evaluate_bounded(
                "['#quote-link', '#attribution-link'].map(id => \
                   getComputedStyle(document.querySelector(id)).textDecorationLine)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(lines, ["underline", "underline"]);
        fixture.close().await.unwrap();
    });
}

/// How far `selector` pokes out past the right edge of its paragraph (a heading: its column), in px.
async fn overflow(page: &chromiumoxide::Page, selector: &str) -> f64 {
    page.evaluate_bounded(format!(
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

/// Todo 2543, 2547, 2589 and 3065: a long word in a heading, body text, a quote and its caption, and the
/// largest key at 320px (WCAG 1.4.10), and every key's text read out in the sentence.
#[test]
fn a_long_title_and_the_keys_stay_inside_the_column() {
    block_on(async {
        let fixture = Fixture::open("/typography", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#title-long").await.unwrap();

        for selector in [
            "#title-long",
            "#text-long",
            "#quote-long",
            "#attribution-long",
            "#quote-caption cite",
            "#kbds kbd:last-of-type",
        ] {
            let past = overflow(page, selector).await;
            assert!(past <= 1.0, "{selector} runs {past}px past its column");
        }

        let tree = ax::snapshot(page, "#kbds").await.unwrap();
        assert_eq!(tree.matches("Ctrl").count(), 6, "the keys' text:\n{tree}");

        fixture.console.assert_clean("the long title").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2985: a long word in a `Paper`'s title stays inside it at 320px.
#[test]
fn a_long_title_stays_inside_its_paper() {
    block_on(async {
        let fixture = Fixture::open("/typography/paper", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#paper-title").await.unwrap();

        let past: f64 = page
            .evaluate_bounded(
                "(() => { const title = document.querySelector('#paper-title'); \
                 const paper = title.parentElement; \
                 const edge = paper.getBoundingClientRect().right \
                     - parseFloat(getComputedStyle(paper).paddingRight); \
                 return Math.max(...[...title.getClientRects()].map(r => r.right)) - edge; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            past <= 1.0,
            "the title runs {past}px past its paper's padding"
        );

        fixture.console.assert_clean("the paper title").unwrap();
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
        page.evaluate_bounded(TEXT_SPACING).await.unwrap();

        // Every element that clips its overflow must still fit its content.
        let clipped: Vec<String> = page
            .evaluate_bounded(
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
