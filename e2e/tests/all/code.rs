//! `Code`: the highlighter's **web** arm, on the one line the two engines split on.
//!
//! Todo 301. `platform/regex.rs` has two `find_impl`s behind one `RegexApi`:
//! the browser's `RegExp` on wasm32, and the `regex` crate everywhere else -
//! which is what every `cargo test`, Blitz and the `fullstack` server run. So
//! every highlighter test there is was about the arm users do not see, and
//! todo 279's fix was confirmed on the web exactly once, by hand, in a browser.
//!
//! `\b` is ASCII in JavaScript and Unicode in the `regex` crate, so `é` ends a
//! word to one engine and not to the other. Before todo 279 this line produced
//! **four** spans natively (`éif` unmarked, so one keyword) and **five** on the
//! web. This test asserts the five, span for span, against the browser - the same
//! list libero's `a_keyword_after_a_non_ascii_letter_tokenizes_as_it_does_on_
//! the_web` asserts against the `regex` crate. One line, both engines, and the
//! next person to touch a grammar or the shorthand rewrite finds out here
//! rather than in a user's hydration mismatch.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::wait;
use e2e::{Fixture, Suite, Viewport, ax};

const COPY: &str = "#diff-block button";
const FLOATING_COPY: &str = "#wide-block button";
const WIDE_SCROLL: &str = "#wide-block [role=region]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("code", "/code")
        .focusable(COPY)
        .focusable(FLOATING_COPY)
        .focusable(WIDE_SCROLL)
        .targets(COPY)
        .targets(FLOATING_COPY)
        .contrast_covers("#diff-block")
        .contrast_covers("#wide-block")
        .run();
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

/// Read off `<code>`'s element children rather than its text: the spans **are**
/// the result. A check on `textContent` would pass against a `Code` that
/// highlighted nothing at all.
const SPANS: &str = r#"(() => {
    const el = document.querySelector('#non-ascii-code');
    if (!el) return [];
    return [...el.children].map(c => [c.textContent, c.getAttribute('class')]);
})()"#;

#[test]
fn a_keyword_after_a_non_ascii_letter_is_highlighted_by_the_browsers_regexp() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();

        // The highlight runs in a `use_resource`, so the first render is the
        // bare source in a text node and there are no element children at all.
        // Waiting on the element would be waiting on nothing.
        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#non-ascii-code')?.children.length > 0",
            "the highlighter to replace the source with spans",
        )
        .await
        .unwrap();

        let spans: Vec<(String, Option<String>)> = fixture
            .page
            .evaluate(SPANS)
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

        const NESTED: &str = r#"[...(document.querySelector('#nested-comment-code')?.children ?? [])]
            .map(c => [c.textContent, c.getAttribute('class')])"#;
        wait::for_js_true(
            &fixture.page,
            &format!("{NESTED}.length > 0"),
            "the highlighter to replace the source with spans",
        )
        .await
        .unwrap();

        let spans: Vec<(String, Option<String>)> = fixture
            .page
            .evaluate(NESTED)
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
