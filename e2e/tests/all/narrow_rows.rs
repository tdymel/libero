//! A flex row narrower than its item's content: the item fits it (WCAG 1.4.10).

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

/// Every part past its row, as `case: TAG.slot left-right`.
const PAST: &str = "[...document.querySelectorAll('[data-case]')].flatMap(row => {
    const box = row.getBoundingClientRect();
    return [...row.querySelectorAll('*')]
        .filter(e => { const b = e.getBoundingClientRect(); return b.width > 1 && (b.right > box.right + 1 || b.left < box.left - 1); })
        .map(e => `${row.dataset.case}: ${e.tagName}.${e.dataset.slot || ''} ${Math.round(e.getBoundingClientRect().left)}-${Math.round(e.getBoundingClientRect().right)}`);
})";

/// Todo 1499: a field, a card and a segmented strip shrink into a 200px row; todo 1507:
/// so does a Video's column capped as its docs say.
#[test]
fn every_item_fits_a_narrow_flex_row() {
    block_on(async {
        let fixture = Fixture::open("/narrow-rows", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "[data-case=segmented] label")
            .await
            .unwrap();
        let past: Vec<String> = page.evaluate(PAST).await.unwrap().into_value().unwrap();
        assert!(past.is_empty(), "past the 200px row: {past:?}");
        fixture.console.assert_clean("narrow rows").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1493: a label wider than its `Button` is cut at its end, not at both.
#[test]
fn an_overlong_button_label_keeps_its_start() {
    block_on(async {
        let fixture = Fixture::open("/narrow-rows", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "[data-case=button] button")
            .await
            .unwrap();
        let edges: Vec<f64> = page
            .evaluate(
                "(() => { const button = document.querySelector('[data-case=button] button'); \
                   const range = document.createRange(); range.selectNodeContents(button); \
                   return [range.getBoundingClientRect().left, button.getBoundingClientRect().left]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            edges[0] >= edges[1],
            "the label starts before its button (text, button): {edges:?}"
        );
        fixture.close().await.unwrap();
    });
}
