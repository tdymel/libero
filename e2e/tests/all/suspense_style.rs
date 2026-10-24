//! Sheets first used under a suspense that reaches the root keep their rules once it
//! resolves: dioxus recreates the app's nodes, the outlet's layer blocks included (todo 4124).

use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Every `lsx-` class under `#loaded` that no rule in `document.styleSheets` names.
const UNSTYLED: &str = "(() => { \
    const texts = []; \
    const walk = (rules) => { for (const r of rules) { texts.push(r.cssText); if (r.cssRules) walk(r.cssRules); } }; \
    for (const s of document.styleSheets) walk(s.cssRules); \
    const css = texts.join('\\n'); \
    const classes = new Set([...document.querySelectorAll('#loaded, #loaded *')] \
        .flatMap(e => [...e.classList]).filter(c => c.startsWith('lsx-'))); \
    return [...classes].filter(c => !css.includes('.' + c)); })()";

#[test]
fn a_sheet_first_used_under_a_root_suspense_keeps_its_rules() {
    block_on(async {
        let fixture = Fixture::open("/suspense-style", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        // Mounted blocks, so the new sheets go in as rules, not `<style>`s.
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('style')].some(s => s.textContent.startsWith('@layer lsx-base{}'))",
            "the layer blocks mounted",
        )
        .await
        .unwrap();
        pointer::click(page, "#load").await.unwrap();
        wait::for_visible(page, "#box").await.unwrap();

        let unstyled: Vec<String> = page
            .evaluate_bounded(UNSTYLED)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(unstyled.is_empty(), "classes without a rule: {unstyled:?}");
        let background: String = page
            .evaluate_bounded("getComputedStyle(document.querySelector('#box')).backgroundColor")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(background, "rgb(41, 42, 43)");

        fixture.console.assert_clean("the suspended page").unwrap();
        fixture.close().await.unwrap();
    });
}
