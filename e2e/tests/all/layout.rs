//! `Box`, `Container`, `Flex`, `Center`, `Float`, `AspectRatio` and `Sidebar`: native
//! semantics through `component`, reflow at 320px and focus rings that no
//! wrapper clips.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("layout", "/layout")
        .focusable("#poly-link")
        .focusable("#poly-button")
        .focusable("#ratio-link")
        .run();
}

/// `component` picks the element, and the caller's `role`/`aria-*`/`id` reach it.
#[test]
fn box_renders_the_asked_tag_with_the_callers_attributes() {
    block_on(async {
        let fixture = Fixture::open("/layout", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#poly-nav").await.unwrap();

        let seen: Vec<String> = page
            .evaluate(
                "['#poly-nav', '#poly-link', '#poly-button', '#container', '#sidebar'].map(s => { \
                   const el = document.querySelector(s); \
                   return [el.tagName, el.getAttribute('aria-label'), el.getAttribute('href'), \
                     el.getAttribute('type'), el.getAttribute('aria-pressed')].join('|'); })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            seen,
            [
                "NAV|Sections|||",
                "A||#container||",
                "BUTTON|||button|false",
                "SECTION|Container|||",
                "ASIDE|Filters|||",
            ]
        );

        fixture.console.assert_clean("polymorphic tags").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Nothing pokes out of the 320px column (WCAG 1.4.10). A `display: contents`
/// box (ScrollArea's content) has an empty rect at the origin, so it is skipped.
#[test]
fn every_wrapper_reflows_inside_320px() {
    block_on(async {
        let fixture = Fixture::open("/layout", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#ratio").await.unwrap();

        let past: Vec<String> = page
            .evaluate(
                "(() => { const column = document.querySelector('#column').getBoundingClientRect(); \
                   return [...document.querySelectorAll('#column *')] \
                     .filter(el => getComputedStyle(el).display !== 'contents') \
                     .filter(el => el.getBoundingClientRect().right > column.right + 1 \
                       || el.getBoundingClientRect().left < column.left - 1) \
                     .map(el => el.tagName + '#' + el.id); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(past.is_empty(), "past the 320px column: {past:?}");

        fixture.close().await.unwrap();
    });
}

/// A nested Flex takes its own direction's defaults, not its ancestor's props.
#[test]
fn a_nested_flex_does_not_inherit_its_ancestors_props() {
    block_on(async {
        let fixture = Fixture::open("/layout", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#inner").await.unwrap();

        let seen: Vec<String> = page
            .evaluate(
                "['#outer', '#inner'].map(s => { const c = getComputedStyle(document.querySelector(s)); \
                   return [c.flexWrap, c.alignItems, c.justifyContent].join('|'); })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(seen, ["nowrap|flex-end|center", "wrap|center|flex-start"]);

        fixture.close().await.unwrap();
    });
}

/// How far the focused element's ring reaches past the nearest ancestor that
/// clips its overflow, in px. Zero or less: the whole stripe is visible.
const RING_CLIPPED: &str = "(() => { const el = document.activeElement; \
    const s = getComputedStyle(el); \
    const reach = parseFloat(s.outlineOffset) + parseFloat(s.outlineWidth); \
    const r = el.getBoundingClientRect(); \
    let clip = el.parentElement; \
    while (clip && getComputedStyle(clip).overflowX === 'visible') clip = clip.parentElement; \
    if (!clip) return -1; \
    const c = clip.getBoundingClientRect(); \
    return Math.max(c.left - (r.left - reach), (r.right + reach) - c.right, \
      c.top - (r.top - reach), (r.bottom + reach) - c.bottom); })()";

/// `AspectRatio` clips its overflow and stretches its child to fill it, so a
/// ring drawn outside the child was cut away entirely (WCAG 2.4.7).
#[test]
fn a_focused_child_of_aspect_ratio_keeps_its_ring() {
    block_on(async {
        let fixture = Fixture::open("/layout", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#ratio-link", 10).await.unwrap();

        let clipped: f64 = page
            .evaluate(RING_CLIPPED)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            clipped <= 0.0,
            "the ring runs {clipped}px past the AspectRatio's clip"
        );

        fixture
            .console
            .assert_clean("the AspectRatio ring")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
