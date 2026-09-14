//! `Scroller`: a strip of buttons with a step control overlaid at each end.

use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, motion};
use e2e::{Fixture, Viewport, wait};

/// How much of the focused item is hidden, in px: under a control that is
/// showing, or outside the strip's clip.
const HIDDEN: &str = "(() => { const a = document.activeElement.getBoundingClientRect(); \
    const v = document.querySelector('[role=region]').getBoundingClientRect(); \
    const under = [...document.querySelectorAll('#strip > button')] \
    .filter(b => getComputedStyle(b).opacity !== '0') \
    .map(b => { const c = b.getBoundingClientRect(); \
    return Math.min(a.right, c.right) - Math.max(a.left, c.left); }); \
    return Math.max(0, v.left - a.left, a.right - v.right, ...under); })()";

/// Waits on the item coming clear, not on a time: a smooth scroll in a
/// background page crawls, but it lands. Without the fix it stays under the
/// control, and the wait gives up.
async fn tab_along(page: &chromiumoxide::Page) {
    keyboard::tab_to(page, "#before", 3).await.unwrap();
    keyboard::tab_to(page, "#tag-0", 3).await.unwrap();
    // To the end and back.
    for i in (1..12).chain((0..11).rev()) {
        match focused_index(page).await > i {
            true => keyboard::press_shift(page, keyboard::TAB).await.unwrap(),
            false => keyboard::press(page, keyboard::TAB).await.unwrap(),
        }
        focus::assert_focused(page, &format!("#tag-{i}"), "Tab along the strip")
            .await
            .unwrap();
        if let Err(error) =
            wait::for_js_true(page, &format!("{HIDDEN} <= 0.5"), &format!("tag-{i} clear")).await
        {
            let hidden: f64 = page.evaluate(HIDDEN).await.unwrap().into_value().unwrap();
            panic!("tag-{i} has {hidden}px hidden: {error}");
        }
    }
}

async fn focused_index(page: &chromiumoxide::Page) -> usize {
    let id: String = page
        .evaluate("document.activeElement.id")
        .await
        .unwrap()
        .into_value()
        .unwrap();
    id.trim_start_matches("tag-").parse().unwrap()
}

/// 2.4.11: Chromium's focus scroll alone left an item 34px under the forward
/// control's fade; it ignores `scroll-padding` and `scroll-margin`.
#[test]
fn a_tabbed_item_is_not_left_under_a_control() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        motion::set_reduced_motion(&fixture.page, true)
            .await
            .unwrap();
        tab_along(&fixture.page).await;
        fixture.close().await.unwrap();
    });
}

/// With smooth scrolling on, Chromium's focus scroll alone sometimes left the
/// strip where it was: the item wholly outside the clip.
#[test]
fn a_tabbed_item_scrolls_into_view_with_smooth_scrolling() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        motion::set_reduced_motion(&fixture.page, false)
            .await
            .unwrap();
        tab_along(&fixture.page).await;
        fixture.close().await.unwrap();
    });
}
