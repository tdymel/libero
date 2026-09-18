//! `Scroller`: a strip of buttons with a step control overlaid at each end.

use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, motion, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

/// The strip's scrolling viewport.
const STRIP: &str = "[role=region]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("scroller", "/scroller")
        .focusable("#tag-0")
        .targets("#strip > button")
        .run();
}

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
/// control's fade; it ignores `scroll-padding` and `scroll-margin`. Reduced,
/// no scroll of the strip is smooth.
#[test]
fn a_tabbed_item_is_not_left_under_a_control() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, true).await.unwrap();
        motion::spy_scrolls(page, STRIP, false).await.unwrap();
        tab_along(page).await;
        let scrolls = motion::scrolls(page).await.unwrap();
        assert!(
            !scrolls.is_empty() && scrolls.iter().all(|scroll| !scroll.smooth),
            "the strip's scrolls under reduced motion: {scrolls:?}"
        );
        fixture.close().await.unwrap();
    });
}

/// The backward control, then the forward one.
const BACK: &str = "#strip > button:first-of-type";
const FORWARD: &str = "#strip > button:last-of-type";

/// Where a control sits against the strip's middle (negative: left of it),
/// and which way its glyph points (`b` of its turn: 1 left, -1 right).
fn side_and_glyph(control: &str) -> String {
    format!(
        "(() => {{ const c = document.querySelector('{control}').getBoundingClientRect(); \
         const v = document.querySelector('[role=region]').getBoundingClientRect(); \
         const glyph = document.querySelector('{control} svg'); \
         return [c.left + c.width / 2 - (v.left + v.width / 2), \
         Math.round(new DOMMatrix(getComputedStyle(glyph).transform).b)]; }})()"
    )
}

/// Under RTL the strip starts at its right edge: the forward control sits on
/// the left and points left, and a step moves `scrollLeft` below 0.
#[test]
fn under_rtl_the_controls_mirror_and_step_to_the_end() {
    block_on(async {
        let fixture = Fixture::open("/scroller/rtl", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{FORWARD}').getAttribute('aria-disabled') === 'false'"
            ),
            "the forward control to offer the end",
        )
        .await
        .unwrap();

        for (control, side, glyph) in [(FORWARD, -1.0, 1.0), (BACK, 1.0, -1.0)] {
            let [at, turn]: [f64; 2] = page
                .evaluate(side_and_glyph(control))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(at * side > 0.0, "{control} sits {at}px off the middle");
            assert_eq!(turn, glyph, "{control}'s glyph points the wrong way");
        }

        pointer::click(page, FORWARD).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=region]').scrollLeft < -1",
            "a forward step to scroll towards the end",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('{BACK}').getAttribute('aria-disabled') === 'false'"),
            "the backward control to offer the start",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("an RTL step").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Under RTL the end is to the left, so dragging the content right reveals it.
#[test]
fn under_rtl_a_drag_to_the_right_heads_for_the_end() {
    block_on(async {
        let fixture = Fixture::open("/scroller/rtl", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{FORWARD}').getAttribute('aria-disabled') === 'false'"
            ),
            "the strip to measure",
        )
        .await
        .unwrap();

        let from = pointer::centre_of(page, STRIP).await.unwrap();
        let to = pointer::Point {
            x: from.x + 120.0,
            y: from.y,
        };
        pointer::drag(page, from, to, 12).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=region]').scrollLeft < -60",
            "the drag to follow the pointer",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("an RTL drag").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 2.4.11 under RTL: an item is measured from the right edge, where the
/// strip starts.
#[test]
fn under_rtl_a_tabbed_item_is_not_left_under_a_control() {
    block_on(async {
        let fixture = Fixture::open("/scroller/rtl", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, true).await.unwrap();
        tab_along(page).await;
        fixture.close().await.unwrap();
    });
}

/// With smooth scrolling on, something can cut the strip's scroll short, and
/// the item stayed hidden: the scroll's end checks again. The suite's
/// Chromium scrolls instantly (687), so every other smooth scroll lands
/// half-way.
#[test]
fn a_tabbed_item_scrolls_into_view_with_smooth_scrolling() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, false).await.unwrap();
        motion::spy_scrolls(page, STRIP, true).await.unwrap();
        tab_along(page).await;
        let scrolls = motion::scrolls(page).await.unwrap();
        assert!(
            scrolls.iter().any(|scroll| scroll.cut),
            "no smooth scroll of the strip was cut short: {scrolls:?}"
        );
        fixture.close().await.unwrap();
    });
}
