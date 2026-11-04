//! Responsive `orientation` (todo 5656): the layout follows the window from CSS, the arrow keys
//! from `use_responsive`.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// The toolbar's `aria-orientation`, its flex direction, the list's display and the focus.
const AXIS: &str = "(() => { const bar = document.getElementById('rso-toolbar'); \
    const list = document.getElementById('rso-list'); \
    return [bar.getAttribute('aria-orientation') || 'horizontal', getComputedStyle(bar).flexDirection, \
    getComputedStyle(list).display, document.activeElement.id].join('|'); })()";

/// The radio group's and marquee track's flex direction, the divider's line side, the slider's
/// `aria-orientation`, the carousel track's direction, the segmented strip's direction and
/// whether the card row grows its cards.
const PARTS: &str = "(() => { const css = (sel) => getComputedStyle(document.querySelector(sel)); \
    return [css('#rso-radio [role=radiogroup]').flexDirection, \
    css('#rso-divider').borderRightWidth === '0px' ? 'horizontal' : 'vertical', \
    css('#rso-marquee > [data-slot=track]').flexDirection, \
    document.querySelector('#rso-slider [role=slider]').getAttribute('aria-orientation') || 'horizontal', \
    css('#rso-carousel [data-slot=track]').flexDirection, \
    css('#rso-segmented [data-slot=control]').flexDirection, \
    css('#rso-cards [role=radiogroup] > *').flexGrow === '1' ? 'grow' : 'fixed'].join('|'); })()";

async fn parts_are(page: &Page, want: &str) {
    let reached = wait::for_js_true(page, &format!("{PARTS} === '{want}'"), want).await;
    let now: String = page
        .evaluate_bounded(PARTS)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(reached.is_ok(), "{now}, want {want}");
}

async fn axis_is(page: &Page, want: &str) {
    let reached = wait::for_js_true(page, &format!("{AXIS} === '{want}'"), want).await;
    let now: String = page
        .evaluate_bounded(AXIS)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(reached.is_ok(), "{now}, want {want}");
}

#[test]
fn a_responsive_orientation_moves_its_layout_and_keys_with_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-orientation", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rso-bold").await.unwrap();
        axis_is(page, "horizontal|row|grid|").await;
        parts_are(page, "row|horizontal|row|horizontal|row|row|grow").await;
        pointer::click(page, "#rso-bold").await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        axis_is(page, "horizontal|row|grid|rso-italic").await;

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        axis_is(page, "vertical|column|flex|rso-italic").await;
        parts_are(page, "column|vertical|column|vertical|column|column|fixed").await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        axis_is(page, "vertical|column|flex|rso-under").await;

        fixture
            .console
            .assert_clean("responsive orientation")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
