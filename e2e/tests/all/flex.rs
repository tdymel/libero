//! `Flex`: `component` picks the element and the flex layout stays.

use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Todo 4102: `Flex` renders the asked tag and still lays its items out in a row.
#[test]
fn flex_renders_a_list_as_ul_with_li_items() {
    block_on(async {
        let fixture = Fixture::open("/flex", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#flex-list").await.unwrap();

        let seen: String = page
            .evaluate_bounded(
                "(() => { const el = document.querySelector('#flex-list'); \
                   const [a, b] = el.children; \
                   return [el.tagName, getComputedStyle(el).display, el.getAttribute('aria-label'), \
                     [...el.children].map(c => c.tagName).join(','), \
                     a.getBoundingClientRect().top === b.getBoundingClientRect().top].join('|'); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(seen, "UL|flex|Tags|LI,LI|true");

        fixture.console.assert_clean("flex as a list").unwrap();
        fixture.close().await.unwrap();
    });
}
