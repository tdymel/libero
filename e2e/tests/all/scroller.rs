//! `Scroller`: a strip of buttons with a step control overlaid at each end.

use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, motion};
use e2e::{Fixture, Viewport};

/// How much of the focused item is hidden, in px: under a control that is
/// showing, or outside the strip's clip.
const HIDDEN: &str = "(() => { const a = document.activeElement.getBoundingClientRect(); \
    const v = document.querySelector('[role=region]').getBoundingClientRect(); \
    const under = [...document.querySelectorAll('#strip > button')] \
    .filter(b => getComputedStyle(b).opacity !== '0') \
    .map(b => { const c = b.getBoundingClientRect(); \
    return Math.min(a.right, c.right) - Math.max(a.left, c.left); }); \
    return Math.max(0, v.left - a.left, a.right - v.right, ...under); })()";

async fn tab_along(page: &chromiumoxide::Page, settle_ms: u32) {
    keyboard::tab_to(page, "#before", 3).await.unwrap();
    keyboard::tab_to(page, "#tag-0", 3).await.unwrap();
    for i in 1..12 {
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::assert_focused(page, &format!("#tag-{i}"), "Tab along the strip")
            .await
            .unwrap();
        page.evaluate(format!(
            "new Promise(r => setTimeout(() => r(1), {settle_ms}))"
        ))
        .await
        .unwrap();
        let hidden: f64 = page.evaluate(HIDDEN).await.unwrap().into_value().unwrap();
        assert!(hidden <= 0.5, "tag-{i} has {hidden}px hidden");
    }
}

/// 2.4.11: an item at the strip's end stays 34px under the forward control's
/// fade; Chromium's focus scroll ignores `scroll-padding` and `scroll-margin`.
#[test]
#[ignore = "Olaf-93 review S1: focused item left under a control"]
fn a_tabbed_item_is_not_left_under_a_control() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        motion::set_reduced_motion(&fixture.page, true)
            .await
            .unwrap();
        tab_along(&fixture.page, 100).await;
        fixture.close().await.unwrap();
    });
}

/// With smooth scrolling on, Tab onto the next item sometimes leaves the strip
/// where it was: the item stays wholly outside the clip.
#[test]
#[ignore = "Olaf-93 review S2: smooth focus scroll does not land"]
fn a_tabbed_item_scrolls_into_view_with_smooth_scrolling() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        motion::set_reduced_motion(&fixture.page, false)
            .await
            .unwrap();
        tab_along(&fixture.page, 900).await;
        fixture.close().await.unwrap();
    });
}
