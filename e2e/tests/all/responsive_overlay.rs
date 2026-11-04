//! Responsive overlay props (todo 5657): popover sides and the drawer's edge follow the window
//! through `use_responsive`, a dialog's full screen through range copies.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Where the tooltip and the hover card sit against their triggers.
const SIDES: &str = "(() => { const side = (pop, trigger) => { \
    const p = document.querySelector(pop)?.getBoundingClientRect(); \
    const t = document.querySelector(trigger).getBoundingClientRect(); \
    if (!p) return 'none'; \
    return p.left >= t.right - 1 ? 'end' : p.top >= t.bottom - 1 ? 'bottom' : 'other'; }; \
    return side('[role=tooltip]:not([hidden])', '#rsv-tip-trigger') + '|' \
    + side('[role=dialog][aria-label=Profile]', '#rsv-card-trigger'); })()";

/// Whether the dialog fills the width.
const DIALOG: &str = "(() => { const d = document.querySelector('[role=dialog]')?.getBoundingClientRect(); \
    if (!d) return 'none'; \
    return d.width >= document.documentElement.clientWidth - 1 ? 'full' : 'centred'; })()";

/// The edge the drawer docks to.
const DRAWER: &str = "(() => { const d = document.querySelector('[role=dialog][aria-label=Panel]')?.getBoundingClientRect(); \
    if (!d) return 'none'; \
    const [w, h] = [document.documentElement.clientWidth, innerHeight]; \
    if (Math.abs(d.bottom - h) < 1 && d.width >= w - 1) return 'bottom'; \
    if (Math.abs(d.right - w) < 1 && d.height >= h - 1) return 'end'; \
    return 'other'; })()";

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
fn a_responsive_popover_side_follows_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-overlay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        reads(page, SIDES, "end|end").await;
        narrow(page).await;
        reads(page, SIDES, "bottom|bottom").await;
        fixture.console.assert_clean("responsive popover").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_responsive_full_screen_dialog_fills_the_window_below_md() {
    block_on(async {
        let fixture = Fixture::open("/responsive-overlay/dialog", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        reads(page, DIALOG, "centred").await;
        narrow(page).await;
        reads(page, DIALOG, "full").await;
        fixture.console.assert_clean("responsive dialog").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_responsive_drawer_anchor_docks_to_the_window_s_edge() {
    block_on(async {
        let fixture = Fixture::open("/responsive-overlay/drawer", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        reads(page, DRAWER, "end").await;
        narrow(page).await;
        reads(page, DRAWER, "bottom").await;
        fixture.console.assert_clean("responsive drawer").unwrap();
        fixture.close().await.unwrap();
    });
}
