//! Sheets registered after mount go into the outlet's layer blocks as rules, so a new
//! class costs no `<style>` and no whole-page restyle (todo 2186).

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, wait};

/// The rules in the outlet's blocks that mention `needle`, and the page's `<style>` count.
async fn rules_and_styles(page: &Page, needle: &str) -> (usize, usize) {
    page.evaluate(format!(
        "(() => {{ const blocks = [...document.querySelectorAll('style')] \
         .find(s => s.textContent.startsWith('@layer lsx-base{{}}')); \
         const rules = blocks ? [...blocks.sheet.cssRules].flatMap(b => [...b.cssRules]) : []; \
         return [rules.filter(r => r.cssText.includes({needle:?})).length, \
         document.querySelectorAll('style').length]; }})()"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

async fn computed(page: &Page, selector: &str, property: &str) -> String {
    page.evaluate(format!(
        "getComputedStyle(document.querySelector({selector:?})).{property}"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

#[test]
fn a_class_mounted_later_is_a_rule_in_its_layer_block() {
    block_on(async {
        let fixture = Fixture::open("/style-rules", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('style')].some(s => s.textContent.startsWith('@layer lsx-base{}'))",
            "the layer blocks mounted",
        )
        .await
        .unwrap();
        let (_, before) = rules_and_styles(page, "rgb(1, 2, 3)").await;

        pointer::click(page, "#toggle").await.unwrap();
        wait::for_visible(page, "#plain").await.unwrap();

        let (rules, styles) = rules_and_styles(page, "rgb(1, 2, 3)").await;
        assert_eq!(rules, 1, "the plain sheet is a rule in the blocks");
        assert_eq!(styles, before, "no sheet got a <style>");
        let (nested, _) = rules_and_styles(page, "rgb(7, 8, 9)").await;
        assert_eq!(nested, 1, "the media sheet is a nested rule in the blocks");
        assert_eq!(
            computed(page, "#plain", "backgroundColor").await,
            "rgb(1, 2, 3)"
        );
        assert_eq!(computed(page, "#media", "color").await, "rgb(7, 8, 9)");
        pointer::hover(page, "#plain").await.unwrap();
        wait::for_js_true(
            page,
            "getComputedStyle(document.querySelector('#plain')).color === 'rgb(4, 5, 6)'",
            "the hover rule applies",
        )
        .await
        .unwrap();

        // Hidden and shown again: the kept rule is reused, not inserted twice.
        pointer::click(page, "#toggle").await.unwrap();
        wait::for_js_true(page, "!document.querySelector('#plain')", "hidden")
            .await
            .unwrap();
        pointer::click(page, "#toggle").await.unwrap();
        wait::for_visible(page, "#plain").await.unwrap();
        assert_eq!(rules_and_styles(page, "rgb(1, 2, 3)").await, (1, before));
        assert_eq!(
            computed(page, "#plain", "backgroundColor").await,
            "rgb(1, 2, 3)"
        );

        fixture.console.assert_clean("the style rules").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A media or container sheet goes in nested (`.a{@media x{..}}`) and matches as its flat
/// `<style>` twin does, in and out of the condition (todo 2231).
#[test]
fn a_nested_query_matches_as_the_flat_one() {
    block_on(async {
        let fixture = Fixture::open("/style-rules", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('style')].some(s => s.textContent.startsWith('@layer lsx-base{}'))",
            "the layer blocks mounted",
        )
        .await
        .unwrap();

        pointer::click(page, "#toggle").await.unwrap();
        wait::for_visible(page, "#container-out").await.unwrap();

        for (id, expected) in [
            ("media", "rgb(7, 8, 9)"),
            ("media-out", "rgb(10, 11, 12)"),
            ("container", "rgb(16, 17, 18)"),
            ("container-out", "rgb(10, 11, 12)"),
        ] {
            let nested = computed(page, &format!("#{id}"), "color").await;
            let flat = computed(page, &format!("#flat-{id}"), "color").await;
            assert_eq!(
                (nested.as_str(), flat.as_str()),
                (expected, expected),
                "#{id}"
            );
        }
        let (nested, _) = rules_and_styles(page, "rgb(16, 17, 18)").await;
        assert_eq!(
            nested, 1,
            "the container sheet is a nested rule in the blocks"
        );

        fixture.console.assert_clean("the nested queries").unwrap();
        fixture.close().await.unwrap();
    });
}
