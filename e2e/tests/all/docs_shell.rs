//! The docs shell's focus move and scroll reset after a navigation, driven
//! through the docs' own file.

use e2e::browser::block_on;
use e2e::wait::PageBounded;
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
            .evaluate_bounded("document.activeElement.tagName")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(focused, "BODY", "the first render moved focus");
        page.evaluate_bounded("document.querySelector('#to-b').click()")
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

/// Todo 5305: a docs page's four tabs fit a phone, so "Accessibility" is not cut off behind a
/// scrollbar. The fixture copies the docs' `doc_tabs_sx`.
#[test]
fn the_four_doc_tabs_fit_a_phone() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    block_on(async {
        let fixture = Fixture::open("/docs-shell/tabs", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "[role=tablist]").await.unwrap();
        for width in [375, 320] {
            page.execute_bounded(SetDeviceMetricsOverrideParams::new(width, 700, 1.0, true))
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!("innerWidth === {width}"),
                "the phone's width",
            )
            .await
            .unwrap();
            let (client, scroll): (i64, i64) = page
                .evaluate_bounded(
                    "(() => { const row = document.querySelector('[role=tablist]');
                        return [row.clientWidth, row.scrollWidth]; })()",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(scroll <= client, "at {width}px: {scroll} in {client}");
        }
        fixture.close().await.unwrap();
    });
}

#[test]
fn the_tab_row_copy_matches_the_docs() {
    let item = |source: &'static str| {
        let start = source.find("fn doc_tabs_sx(").expect("the copied fn");
        &source[start..start + source[start..].find("\n}\n").expect("its end")]
    };
    assert_eq!(
        item(include_str!("../../fixtures/src/docs_shell.rs")),
        item(include_str!("../../../docs/src/components/doc_page.rs")),
    );
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
        page.evaluate_bounded(
            "document.querySelectorAll('#page-area, #page-area *').forEach((e) => e.scrollTop = 600)",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &format!("{AREA_SCROLL} >= 600"), "the area to scroll")
            .await
            .unwrap();
        page.evaluate_bounded("document.querySelector('#to-b').click()")
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
        page.evaluate_bounded("document.querySelector('#to-far').click()")
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

/// A refused fullscreen draws the panel over the whole page: the page area rises
/// above the sticky header, so the Exit under it takes the tap (5040).
#[test]
fn a_drawn_fullscreen_covers_the_docs_header() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/fullscreen", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#panel[data-refused] #toggle")
            .await
            .unwrap();
        page.evaluate_bounded("document.querySelector('#toggle').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#panel')?.dataset.fullscreen === 'drawn'",
            "the refused request to draw the panel",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "(() => { const r = document.querySelector('#toggle').getBoundingClientRect(); \
             return r.top < 96 && document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2)?.closest('#toggle') !== null; })()",
            "Exit, under the header's band, to take the tap",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("the fullscreen fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
