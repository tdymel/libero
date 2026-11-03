//! Responsive axis props (todo 5643): the copies registered on first use switch the look at
//! each breakpoint, before and after mount, and `hidden_from` / `visible_from` hide by width.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// The buttons' `height fill`, then the hidden button's and the badge's `display`, blockified by the `Flex`,
/// then the icon's and the chip's heights.
const LOOK: &str = "(() => { const s = id => getComputedStyle(document.getElementById(id)); \
    const box = id => s(id).height + (s(id).backgroundColor === 'rgba(0, 0, 0, 0)' ? ' clear' : ' fill'); \
    return [box('rsp-button'), box('rsp-late'), s('rsp-hidden').display, s('rsp-badge').display, \
    s('rsp-icon').height, s('rsp-chip').height].join('|'); })()";

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
        assert_eq!(look(page).await, "60px fill|50px fill|none|flex|60px|32px");

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let narrow = "30px clear|24px fill|flex|none|30px|24px";
        let reached =
            wait::for_js_true(page, &format!("{LOOK} === '{narrow}'"), "the narrow look").await;
        assert!(reached.is_ok(), "{}", look(page).await);

        fixture.console.assert_clean("responsive props").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The strip's `aria-orientation`, its flex direction and the selected tab.
const STRIP: &str = "(() => { const list = document.getElementById('rsp-tabs-tablist'); \
    const selected = list.querySelector('[aria-selected=\"true\"]').id; \
    return [list.getAttribute('aria-orientation'), getComputedStyle(list).flexDirection, selected].join('|'); })()";

async fn strip_is(page: &Page, want: &str) {
    let reached = wait::for_js_true(page, &format!("{STRIP} === '{want}'"), want).await;
    let now: String = page
        .evaluate_bounded(STRIP)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(reached.is_ok(), "{now}, want {want}");
}

/// Todo 5653: a column with Up and Down from 62rem, a row with Left and Right under it.
#[test]
fn a_responsive_tabs_placement_moves_its_keys_with_the_layout() {
    block_on(async {
        let fixture = Fixture::open("/responsive", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rsp-tabs-tab-0").await.unwrap();
        strip_is(page, "vertical|column|rsp-tabs-tab-0").await;
        pointer::click(page, "#rsp-tabs-tab-0").await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        strip_is(page, "vertical|column|rsp-tabs-tab-1").await;

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        strip_is(page, "horizontal|row|rsp-tabs-tab-1").await;
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        strip_is(page, "horizontal|row|rsp-tabs-tab-2").await;

        fixture.console.assert_clean("responsive tabs").unwrap();
        fixture.close().await.unwrap();
    });
}
