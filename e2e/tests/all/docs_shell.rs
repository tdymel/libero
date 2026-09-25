//! The docs shell's focus move and scroll reset after a navigation, driven
//! through the docs' own file.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

#[test]
fn a_navigation_focuses_the_new_heading_but_a_load_does_not() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/heading", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#to-b").await.unwrap();
        let focused: String = page
            .evaluate("document.activeElement.tagName")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(focused, "BODY", "the first render moved focus");
        page.evaluate("document.querySelector('#to-b').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.tagName === 'H1' && document.activeElement.textContent === 'Page B'",
            "focus on the new heading",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the heading fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The largest `scrollTop` in the fixture's area: whichever box scrolls.
const AREA_SCROLL: &str = "Math.max(...[document.querySelector('#page-area'), \
    ...document.querySelectorAll('#page-area *')].map((e) => e.scrollTop))";

#[test]
fn a_navigation_starts_the_new_page_at_the_top() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/scroll", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#to-b").await.unwrap();
        page.evaluate(
            "document.querySelectorAll('#page-area, #page-area *').forEach((e) => e.scrollTop = 600)",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &format!("{AREA_SCROLL} >= 600"), "the area to scroll")
            .await
            .unwrap();
        page.evaluate("document.querySelector('#to-b').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('#page-area h1').textContent === 'Page B' && {AREA_SCROLL} === 0"),
            "the new page at the top",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the scroll fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1170: a section link lands its section near the area's top, focus on its heading.
#[test]
fn a_section_navigation_lands_on_the_section() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/section", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#to-far").await.unwrap();
        page.evaluate("document.querySelector('#to-far').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "(() => { const far = document.querySelector('#far'); \
               if (!far || document.activeElement !== far.firstElementChild) return false; \
               const gap = far.getBoundingClientRect().top \
                 - document.querySelector('#page-area').getBoundingClientRect().top; \
               return gap >= 0 && gap <= 40; })()",
            "the far section at the top, its heading focused",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the section fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1175: a deep link's fragment lands on its section on a fresh load, and stays in
/// the URL for a reload (1184).
#[test]
fn a_fragment_load_lands_on_the_section() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/fragment", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "(() => { const far = document.querySelector('#far'); \
               if (!far || document.activeElement !== far.firstElementChild) return false; \
               const gap = far.getBoundingClientRect().top \
                 - document.querySelector('#page-area').getBoundingClientRect().top; \
               return gap >= 0 && gap <= 40; })()",
            "the far section at the top, its heading focused",
        )
        .await
        .unwrap();
        wait::for_js_true(page, "location.hash === '#far'", "the fragment in the URL")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("the fragment fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
