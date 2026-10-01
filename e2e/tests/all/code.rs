//! `Code`: the highlighter's web (`RegExp`) arm (301). `\b` is ASCII in JS and Unicode in
//! `regex` (279): the same five spans libero's native test asserts, here in the browser.

use e2e::browser::block_on;
use e2e::passes::{contrast, keyboard};
use e2e::wait;
use e2e::{Fixture, Suite, Viewport, ax};

const COPY: &str = "#diff-block button";
const FLOATING_COPY: &str = "#wide-block button";
const WIDE_SCROLL: &str = "#wide-block [role=region]";
const TALL_COPY: &str = "#tall-block button";
const TALL_SCROLL: &str = "#tall-block [role=region]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("code", "/code")
        .waive(contrast::LINE_NUMBERS)
        .focusable(COPY)
        .focusable(FLOATING_COPY)
        .focusable(WIDE_SCROLL)
        .focusable(TALL_COPY)
        .focusable(TALL_SCROLL)
        .targets(COPY)
        .targets(FLOATING_COPY)
        .targets(TALL_COPY)
        .contrast_covers("#diff-block")
        .contrast_covers("#wide-block")
        .run();
}

/// Todo 1652: a classic vertical scrollbar takes width off the scroll box; the floating
/// copy button sits left of it, not over it. Headless Chrome scrollbars, so a 16px right padding stands in for the gutter: it narrows the
/// `pre` as a scrollbar does, which is what the block measures.
#[test]
fn the_floating_copy_button_clears_the_vertical_scrollbar() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, TALL_COPY).await.unwrap();
        page.evaluate(format!(
            "document.querySelector('{TALL_SCROLL}').style.paddingRight = '16px'"
        ))
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const box = document.querySelector('{TALL_SCROLL}').getBoundingClientRect(); \
                 return document.querySelector('{TALL_COPY}').getBoundingClientRect().right <= box.right - 16 - 8 + 0.5; }})()"
            ),
            "the copy button 8px left of the gutter",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `[the code's direction, whether its first token starts at the scroller's
/// left, whether the header's copy button sits left of its label]`.
const RTL_LAYOUT: &str = "(() => { \
    const block = document.querySelector('#diff-block'); \
    const scroller = block.querySelector('[dir=ltr]'); \
    const code = scroller.querySelector('code'); \
    const token = code.querySelector('span'); \
    const label = block.querySelector('span').getBoundingClientRect(); \
    const copy = block.querySelector('button').getBoundingClientRect(); \
    return [getComputedStyle(code).direction, \
        Math.abs(token.getBoundingClientRect().left - scroller.getBoundingClientRect().left) < 64, \
        copy.right <= label.left]; })()";

/// Todo 735: code stays LTR on an RTL page; the header around it still mirrors.
#[test]
fn code_stays_ltr_on_an_rtl_page() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.evaluate("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        let (direction, at_left, mirrored): (String, bool, bool) = page
            .evaluate(RTL_LAYOUT)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(direction, "ltr", "the code follows the page");
        assert!(at_left, "the code does not start at the left");
        assert!(mirrored, "the header's copy button is not at the RTL end");
        fixture.close().await.unwrap();
    });
}

/// A keyboard copy is announced through the always-mounted status, and focus
/// stays on the button, whose name does not change under it.
#[test]
fn a_keyboard_copy_is_announced() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, COPY, 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#diff-block [role=status]').textContent === 'Copied'",
            "the copy to be announced",
        )
        .await
        .unwrap();
        let name: String = page
            .evaluate("document.activeElement.getAttribute('aria-label')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(name, "Copy code");
        fixture.console.assert_clean("a keyboard copy").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1025: a block is a group named by its label, and its copy button, still "Copy code",
/// is described by it. A scrolling block's region keeps the language name inside.
#[test]
fn a_block_is_a_group_that_describes_its_copy_button() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "!!document.querySelector('#wide-block [role=region]')",
            "the wide block to become a scroll region",
        )
        .await
        .unwrap();

        let plain = ax::snapshot(page, "#diff-block").await.unwrap();
        assert!(plain.starts_with("group \"Rust code\"\n"), "{plain}");
        assert!(plain.contains("button \"Copy code\""), "{plain}");
        assert_eq!(ax::description(page, COPY).await.unwrap(), "Rust code");

        let labelled = ax::snapshot(page, "#wide-block").await.unwrap();
        assert!(
            labelled.starts_with("group \"The greeting, Rust code\"\n"),
            "{labelled}"
        );
        assert!(labelled.contains("region \"Rust code\""), "{labelled}");
        assert_eq!(
            ax::description(page, FLOATING_COPY).await.unwrap(),
            "The greeting, Rust code"
        );

        fixture.console.assert_clean("the code fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `<code>`'s pieces, a classed span or bare text each: the spans **are** the
/// result, which a check on `textContent` would pass without.
const PIECES: &str = r#"[...(document.querySelector('#ID')?.childNodes ?? [])]
    .filter(c => (c.nodeType === 1 || c.nodeType === 3) && c.textContent)
    .map(c => [c.textContent, c.nodeType === 1 ? c.getAttribute('class') : null])"#;

/// The pieces of the `<code>` with this id.
fn pieces(id: &str) -> String {
    PIECES.replace("ID", id)
}

#[test]
fn a_keyword_after_a_non_ascii_letter_is_highlighted_by_the_browsers_regexp() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();

        // The highlight runs in a `use_resource`: the first render is a bare text node,
        // with no element to wait on.
        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#non-ascii-code')?.children.length > 0",
            "the highlighter to replace the source with spans",
        )
        .await
        .unwrap();

        let spans: Vec<(String, Option<String>)> = fixture
            .page
            .evaluate(pieces("non-ascii-code"))
            .await
            .unwrap()
            .into_value()
            .unwrap();

        let expected: Vec<(String, Option<String>)> = vec![
            ("é".into(), None),
            ("if".into(), Some("lsx-tok-keyword".into())),
            (" x; ".into(), None),
            ("if".into(), Some("lsx-tok-keyword".into())),
            (" y".into(), None),
        ];

        assert_eq!(
            spans, expected,
            "the web arm tokenized the non-ASCII line differently from the `regex` arm"
        );

        fixture.console.assert_clean("the code fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 434. A nested block comment ends at the `*/` that balances its opener.
#[test]
fn a_nested_block_comment_is_one_comment_on_the_web() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();

        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#nested-comment-code')?.children.length > 0",
            "the highlighter to replace the source with spans",
        )
        .await
        .unwrap();

        let spans: Vec<(String, Option<String>)> = fixture
            .page
            .evaluate(pieces("nested-comment-code"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let expected: Vec<(String, Option<String>)> = vec![
            ("é ".into(), None),
            ("/* a /* b */ c */".into(), Some("lsx-tok-comment".into())),
            (" x".into(), None),
        ];
        assert_eq!(spans, expected);

        fixture.console.assert_clean("the code fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 595. The highlighted rows stay inside `pre > code`, so the block keeps
/// its `code` role once the highlighter replaces the plain source.
#[test]
fn a_highlighted_code_block_keeps_its_code_role() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelectorAll('#numbered-block pre > code > span').length === 2",
            "the highlighter to draw the rows inside pre > code",
        )
        .await
        .unwrap();
        let snapshot = ax::snapshot(page, "#numbered-block").await.unwrap();
        assert!(
            snapshot.lines().any(|line| line.trim() == "code"),
            "{snapshot}"
        );
        fixture.console.assert_clean("the code fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 594. A diff line is not colour alone: a visible `+`/`-` that is hidden
/// from readers and from a selection, and a hidden word that is read.
#[test]
fn a_diff_line_says_added_or_removed() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        const MARKERS: &str = r#"[...document.querySelectorAll('#diff-block pre > code > span > [aria-hidden=true]')]
            .map(m => m.textContent + (getComputedStyle(m).userSelect === 'none' ? '' : '!')).join('')"#;
        // Line number, then marker, per row.
        wait::for_js_true(
            page,
            &format!("{MARKERS} === '1 2-3+4 '"),
            "a marker per row, unselectable",
        )
        .await
        .unwrap();

        let snapshot = ax::snapshot(page, "#diff-block").await.unwrap();
        let text: Vec<&str> = snapshot.lines().map(str::trim).collect();
        let at = |needle: &str| text.iter().position(|line| *line == needle).unwrap();
        assert_eq!(
            at(r#"StaticText "Removed""#) + 2,
            at(r#"StaticText "old""#),
            "{snapshot}"
        );
        assert_eq!(
            at(r#"StaticText "Added""#) + 2,
            at(r#"StaticText "new""#),
            "{snapshot}"
        );
        assert!(!snapshot.contains(r#""+""#), "{snapshot}");

        fixture.console.assert_clean("the code fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 668. With no language the rows, markers and bars still come, and the
/// scroll region takes its name from the localization.
#[test]
fn a_diff_without_a_language_is_still_marked() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        const ROWS: &str = r#"[...document.querySelectorAll('#plain-diff-block pre > code > span')]
            .map(r => (r.querySelector('[aria-hidden=true]')?.textContent ?? '') + ':' + getComputedStyle(r).boxShadow.includes('inset')).join()"#;
        wait::for_js_true(
            page,
            &format!("{ROWS} === ' :true,-:true,+:true'"),
            "a marker and a bar per marked row",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector(\"#wide-block [role=region]\")?.getAttribute('aria-label') === 'Rust code'",
            "the region named from the localization",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the code fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 594. A marked line keeps its left bar in forced colours, which drop
/// the wash and the inset shadow that draw it otherwise.
#[test]
fn a_highlighted_line_keeps_its_bar_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        const BAR: &str = "(() => { const rows = [...document.querySelectorAll('#wide-block pre > code > span')]; \
            return rows.length === 3 && rows.map(r => getComputedStyle(r).borderLeftWidth).join() === '0px,3px,0px'; })()";
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(page, BAR, "a bar on line 2 only")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1538. A highlighted multi-line source keeps its line break, as the
/// plain one does, so `1;let` is not read as one word.
#[test]
fn a_highlighted_multi_line_code_keeps_its_line_break() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        wait::for_js_true(
            &fixture.page,
            "(() => { const c = document.querySelector('#multi-line-code'); \
             return c?.children.length > 0 && c.textContent === 'let a = 1;\\nlet b = 2;'; })()",
            "the highlighted spans with the line break between them",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 732. A short span stays whole even where `overflow-wrap: anywhere`
/// lowered its min-content; a long one still wraps inside 320px (1.4.10).
#[test]
fn a_short_code_span_keeps_its_token_whole() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let lines: i64 = page
            .evaluate("document.querySelector('#short-code').getClientRects().length")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(lines, 1, "the short span broke across lines");

        let overflow: bool = page
            .evaluate(
                "(() => { const t = document.querySelector('#long-code-text'); \
                 return t.scrollWidth > t.clientWidth \
                 || document.querySelector('#long-code').getClientRects().length < 2; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!overflow, "the long span ran out of its 320px column");
        fixture.close().await.unwrap();
    });
}

/// Todo 771. Without a gutter a blank line keeps a full row's height.
#[test]
fn a_blank_line_keeps_its_row_without_line_numbers() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();
        const HEIGHTS: &str = r#"[...document.querySelectorAll('#blank-line-block pre > code > span')]
            .map(r => Math.round(r.getBoundingClientRect().height)).join()"#;
        wait::for_js_true(
            &fixture.page,
            &format!("{HEIGHTS} === '20,20,20'"),
            "three 20px rows, the blank one included",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 424. The gutter is drawn only once highlighting resolves, which SSR
/// never sees; read aloud, its numbers interleave with the code.
#[test]
fn a_code_blocks_line_numbers_are_hidden_from_readers() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();

        // A row is a `code` child holding exactly the gutter span and the content span.
        const GUTTER: &str = r#"[...document.querySelectorAll('#numbered-block pre > code > span')]
            .filter(d => d.children.length === 2 && [...d.children].every(c => c.tagName === 'SPAN'))
            .map(d => [d.children[0].textContent, d.children[0].getAttribute('aria-hidden')])"#;
        wait::for_js_true(
            &fixture.page,
            &format!("{GUTTER}.length === 2"),
            "the highlighter to draw the gutter",
        )
        .await
        .unwrap();

        let gutter: Vec<(String, Option<String>)> = fixture
            .page
            .evaluate(GUTTER)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let hidden = Some("true".to_string());
        assert_eq!(
            gutter,
            vec![("1".into(), hidden.clone()), ("2".into(), hidden)]
        );

        fixture.console.assert_clean("the code fixture").unwrap();
        fixture.close().await.unwrap();
    });
}
