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
use e2e::wait;
use e2e::{Fixture, Viewport};

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

/// Todo 424. The gutter is drawn only once highlighting resolves, which SSR
/// never sees; read aloud, its numbers interleave with the code.
#[test]
fn a_code_blocks_line_numbers_are_hidden_from_readers() {
    block_on(async {
        let fixture = Fixture::open("/code", Viewport::Desktop).await.unwrap();

        // A row is a `div` holding exactly the gutter span and the content span.
        const GUTTER: &str = r#"[...document.querySelectorAll('#numbered-block div')]
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
