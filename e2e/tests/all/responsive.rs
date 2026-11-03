//! Responsive axis props (todo 5643): the copies registered on first use switch the look at
//! each breakpoint, before and after mount, and `hidden_from` / `visible_from` hide by width.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// The buttons' `height fill`, then the hidden button's and the badge's `display`, blockified by the `Flex`.
const LOOK: &str = "(() => { const s = id => getComputedStyle(document.getElementById(id)); \
    const box = id => s(id).height + (s(id).backgroundColor === 'rgba(0, 0, 0, 0)' ? ' clear' : ' fill'); \
    return [box('rsp-button'), box('rsp-late'), s('rsp-hidden').display, s('rsp-badge').display].join('|'); })()";

async fn look(page: &Page) -> String {
    page.evaluate_bounded(LOOK)
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Large and filled from 62rem, small and outlined under it; a copy inserted after mount too.
#[test]
fn responsive_props_follow_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/responsive", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rsp-button").await.unwrap();
        pointer::click(page, "#rsp-add").await.unwrap();
        wait::for_visible(page, "#rsp-late").await.unwrap();
        assert_eq!(look(page).await, "60px fill|50px fill|none|flex");

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let narrow = "30px clear|24px fill|flex|none";
        let reached =
            wait::for_js_true(page, &format!("{LOOK} === '{narrow}'"), "the narrow look").await;
        assert!(reached.is_ok(), "{}", look(page).await);

        fixture.console.assert_clean("responsive props").unwrap();
        fixture.close().await.unwrap();
    });
}
