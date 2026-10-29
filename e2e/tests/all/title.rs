//! `Title`: a word wider than the column wraps instead of widening the page.

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Todo 4852 (WCAG 1.4.10): at 375px the h1 and the Dialog's h2 end inside the viewport.
#[test]
fn a_long_word_wraps_inside_a_phone() {
    block_on(async {
        let fixture = Fixture::open("/title", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetDeviceMetricsOverrideParams::new(375, 780, 1.0, true))
            .await
            .unwrap();
        wait::for_visible(page, "#long").await.unwrap();
        let js = "(() => { const fits = (sel) => { const e = document.querySelector(sel); \
                  return !!e && e.getBoundingClientRect().right <= innerWidth && e.scrollWidth <= e.clientWidth; }; \
                  return innerWidth === 375 && fits('h1#long') && fits('#dialog h2') \
                    && document.documentElement.scrollWidth <= 375; })()";
        wait::for_js_true(page, js, "a long title within 375px")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}
