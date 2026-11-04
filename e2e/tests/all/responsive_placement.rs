//! Placement per breakpoint (todo 5692): a tour step's side and a floating window's placement
//! follow the window through `use_responsive`.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Where the tour card sits against its target.
const CARD: &str = "(() => { const c = document.querySelector('[role=dialog]')?.getBoundingClientRect(); \
    const t = document.querySelector('#rsp-target').getBoundingClientRect(); \
    if (!c || !c.width) return 'none'; \
    return c.left >= t.right - 1 ? 'end' : c.top >= t.bottom - 1 ? 'bottom' : 'other'; })()";

/// The window's quarter of the viewport.
const WINDOW: &str = "(() => { const d = document.querySelector('[role=dialog]')?.getBoundingClientRect(); \
    if (!d || !d.width) return 'none'; \
    const [x, y] = [d.left + d.width / 2, d.top + d.height / 2]; \
    const [w, h] = [document.documentElement.clientWidth / 2, innerHeight / 2]; \
    return x > w && y > h ? 'bottom-end' : x < w && y < h ? 'top-start' : 'other'; })()";

async fn reads(page: &Page, probe: &str, want: &str) {
    let reached = wait::for_js_true(page, &format!("{probe} === '{want}'"), want).await;
    let now: String = page
        .evaluate_bounded(probe)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(reached.is_ok(), "{now}, want {want}");
}

async fn narrow(page: &Page) {
    page.execute_bounded(SetDeviceMetricsOverrideParams::new(900, 800, 1.0, false))
        .await
        .unwrap();
}

#[test]
fn a_responsive_tour_side_follows_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-placement/tour", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "#rsp-start").await.unwrap();
        reads(page, CARD, "end").await;
        narrow(page).await;
        reads(page, CARD, "bottom").await;
        fixture.console.assert_clean("responsive tour").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_responsive_window_placement_follows_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-placement/window", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        reads(page, WINDOW, "bottom-end").await;
        narrow(page).await;
        reads(page, WINDOW, "top-start").await;
        fixture.console.assert_clean("responsive window").unwrap();
        fixture.close().await.unwrap();
    });
}
