//! `NavLink`'s `scroll_into_view` scrolls its sidebar, not the page, just far
//! enough to show the link and its `scroll-margin` (todo 468 N3 moved the
//! `nearest` arithmetic into a function Blitz shares).

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

/// The link's bottom plus its 8rem margin meets the sidebar's bottom.
const SHOWN_NEAREST: &str = "(() => { \
    const bar = document.querySelector('#sidebar'); \
    const link = document.querySelector('#here'); \
    const margin = parseFloat(getComputedStyle(link).scrollMarginBottom); \
    const gap = bar.getBoundingClientRect().bottom - (link.getBoundingClientRect().bottom + margin); \
    return bar.scrollTop > 0 && Math.abs(gap) < 2 && window.scrollY === 0; \
})()";

#[test]
fn an_active_link_scrolls_its_sidebar_just_far_enough() {
    block_on(async {
        let fixture = Fixture::open("/nav-link", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(page, SHOWN_NEAREST, "the link at the sidebar's bottom edge")
            .await
            .unwrap();

        fixture.console.assert_clean("an active nav link").unwrap();
        fixture.close().await.unwrap();
    });
}
