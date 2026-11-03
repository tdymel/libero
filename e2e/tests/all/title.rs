//! `Title`: a word wider than the column wraps instead of widening the page; a cut title
//! keeps its line in place and its link's ring whole when the link takes focus.

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, js, wait};

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

/// Todos 5462, 5463: Tab to a link inside a cut title scrolls nothing, and its ring is inset.
#[test]
fn a_focused_link_in_a_cut_title_keeps_the_line_and_its_ring() {
    block_on(async {
        let fixture = Fixture::open("/title/cut", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        for cut in ["start", "end"] {
            keyboard::tab_to(page, &format!("#link-{cut}"), 4)
                .await
                .unwrap();
            let (overflow, scrolled, offset, shadow): (String, f64, f64, String) = js(
                page,
                format!(
                    "(() => {{ const t = document.getElementById('cut-{cut}'), \
                     s = getComputedStyle(document.getElementById('link-{cut}')); \
                     return [getComputedStyle(t).overflowX, t.scrollLeft, \
                       parseFloat(s.outlineOffset), s.boxShadow]; }})()"
                ),
            )
            .await;
            assert_eq!(overflow, "clip", "{cut}");
            assert_eq!(scrolled, 0.0, "{cut}: focus scrolled the line");
            assert!(offset < 0.0, "{cut}: outline offset {offset}");
            assert!(shadow.contains("inset"), "{cut}: {shadow}");
        }
        fixture.console.assert_clean("a cut title").unwrap();
        fixture.close().await.unwrap();
    });
}
