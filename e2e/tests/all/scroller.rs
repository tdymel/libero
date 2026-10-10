//! `Scroller`: a strip of buttons with a step control overlaid at each end.

use anyhow::{Result, bail};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
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
/// showing, or outside the strip's clip. A control's fade is finished at once:
/// a tab behind runs it at ~1 frame/s.
const HIDDEN: &str = "(() => { const a = document.activeElement.getBoundingClientRect(); \
    const v = document.querySelector('[role=region]').getBoundingClientRect(); \
    const under = [...document.querySelectorAll('#strip > button')] \
    .filter(b => (b.getAnimations().forEach(t => t.finish()), getComputedStyle(b).opacity !== '0')) \
    .map(b => { const c = b.getBoundingClientRect(); \
    return Math.min(a.right, c.right) - Math.max(a.left, c.left); }); \
    return Math.max(0, v.left - a.left, a.right - v.right, ...under); })()";

/// Waits on the item coming clear, not on a time: a smooth background scroll crawls but
/// lands. Without the fix it stays under the control.
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

/// 2.4.11: Chromium's focus scroll ignores `scroll-padding`/`scroll-margin` and left an item
/// 34px under the fade. Reduced, no strip scroll is smooth.
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

/// Px of the focused item's outset ring the strip's clip cuts, per side: `[left, top, right,
/// bottom]`. The ring reaches the halo spread past the item's box.
const RING_CUT: &str = "(() => { const e = document.activeElement; \
    const p = document.createElement('div'); \
    p.style.cssText = 'position: absolute; width: var(--lsx-focus-ring-halo-spread)'; \
    document.body.append(p); const spread = p.getBoundingClientRect().width; p.remove(); \
    if (!(spread > 0)) throw new Error('no ring spread'); \
    const a = e.getBoundingClientRect(); \
    const v = document.querySelector('[role=region]'); const r = v.getBoundingClientRect(); \
    const clip = { left: r.left + v.clientLeft, top: r.top + v.clientTop, \
    right: r.left + v.clientLeft + v.clientWidth, bottom: r.top + v.clientTop + v.clientHeight }; \
    return [clip.left - (a.left - spread), clip.top - (a.top - spread), \
    (a.right + spread) - clip.right, (a.bottom + spread) - clip.bottom].map(c => Math.max(0, c)); })()";

/// Todo 2043: the strip's clip cut the first and last items' outset focus rings (2.4.7).
#[test]
fn an_edge_items_focus_ring_is_not_clipped() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, true).await.unwrap();
        keyboard::tab_to(page, "#before", 3).await.unwrap();
        for (tag, sides) in [("#tag-0", [0, 1, 3]), ("#tag-11", [1, 2, 3])] {
            keyboard::tab_to(page, tag, 12).await.unwrap();
            wait::for_js_true(page, &format!("{HIDDEN} <= 0.5"), "the item in view")
                .await
                .unwrap();
            let cut: [f64; 4] = page.evaluate(RING_CUT).await.unwrap().into_value().unwrap();
            for side in sides {
                assert!(cut[side] <= 0.5, "{tag}: ring cut [l, t, r, b] {cut:?}");
            }
        }
        fixture.close().await.unwrap();
    });
}

/// Todo 734: Blitz scrolled nothing on Tab and fired no `focusin`, so a tabbed
/// item stayed out of the strip or under the forward control.
async fn tab_clears_the_control<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#before").await?;
    for _ in 0..10 {
        d.press(keyboard::TAB).await?;
        if d.is_focused("#tag-8").await? {
            break;
        }
    }
    eventually_focused(d, "#tag-8", "Tab along the strip").await?;
    let cleared = eventually(d, "tag-8 clear of the forward control", async |d| {
        let strip = d.rect(STRIP).await?;
        let tag = d.rect("#tag-8").await?;
        let forward = d.rect(FORWARD).await?;
        Ok(tag.x + tag.width <= forward.x + 1.0 && tag.x >= strip.x - 1.0)
    })
    .await;
    // Todo 2873: seen red on desktop only under load, never reproduced; says where things stopped.
    if let Err(error) = cleared {
        let (focused, strip, tag, forward) = (
            d.focused_id().await?,
            d.rect(STRIP).await?,
            d.rect("#tag-8").await?,
            d.rect(FORWARD).await?,
        );
        bail!("{error}: focus on {focused:?}, strip {strip:?}, tag-8 {tag:?}, forward {forward:?}");
    }
    Ok(())
}

e2e::scenario!(
    a_tabbed_item_clears_the_forward_control,
    "/scroller",
    tab_clears_the_control
);

/// Todo 1625: the strip is a tab stop only while it overflows with nothing focusable
/// inside; a strip that fits, or one of buttons, adds none.
#[test]
fn the_strip_is_a_tab_stop_only_while_it_overflows() {
    block_on(async {
        let fixture = Fixture::open("/scroller/plain", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let wide = "#wide > [data-slot=viewport]";
        let narrow = "#narrow > [data-slot=viewport]";
        wait::for_js_true(
            page,
            &format!("document.querySelector('{wide}').getAttribute('tabindex') === '0'"),
            "the overflowing strip to become a tab stop",
        )
        .await
        .unwrap();
        let narrow_stop: Option<String> = page
            .evaluate(format!(
                "document.querySelector('{narrow}').getAttribute('tabindex')"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(narrow_stop.as_deref(), Some("-1"), "a strip that fits");
        keyboard::tab_to(page, "#before", 3).await.unwrap();
        keyboard::tab_to(page, wide, 3).await.unwrap();
        // The forward control, then After: no stop on the strip that fits.
        keyboard::tab_to(page, "#after", 2).await.unwrap();
        fixture.console.assert_clean("plain strips").unwrap();
        fixture.close().await.unwrap();

        // A strip of buttons: they take the focus, the strip none.
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#before", 3).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::assert_focused(page, "#tag-0", "Tab from Before")
            .await
            .unwrap();
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

/// Todo 2843: a control the mouse stepped to the end keeps the focus, so it stays shown.
#[test]
fn a_clicked_control_stays_shown_at_the_end_of_the_strip() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, true).await.unwrap();
        let ended =
            format!("document.querySelector('{FORWARD}').getAttribute('aria-disabled') === 'true'");
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{FORWARD}').getAttribute('aria-disabled') === 'false'"
            ),
            "the forward control to offer the end",
        )
        .await
        .unwrap();
        for _ in 0..30 {
            let done: bool = page
                .evaluate(ended.as_str())
                .await
                .unwrap()
                .into_value()
                .unwrap();
            if done {
                break;
            }
            pointer::click(page, FORWARD).await.unwrap();
        }
        wait::for_js_true(page, &ended, "the forward control to reach the end")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const b = document.querySelector('{FORWARD}'); \
                 b.getAnimations().forEach(t => t.finish()); \
                 return document.activeElement === b \
                 && getComputedStyle(b).visibility === 'visible' \
                 && getComputedStyle(b).opacity === '1'; }})()"
            ),
            "the clicked control to stay focused and shown",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
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

/// With smooth scrolling a cut-short scroll left the item hidden, so the scroll's end checks
/// again. The suite's Chromium scrolls instantly otherwise (687).
#[test]
fn a_tabbed_item_scrolls_into_view_with_smooth_scrolling() {
    block_on(async {
        let fixture = Fixture::open("/scroller", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, false).await.unwrap();
        motion::spy_scrolls(page, STRIP, true).await.unwrap();
        // In front: a tab behind runs a smooth scroll at ~1 frame/s.
        e2e::frames::in_front(page, async {
            tab_along(page).await;
            Ok(())
        })
        .await
        .unwrap();
        let scrolls = motion::scrolls(page).await.unwrap();
        assert!(
            scrolls.iter().any(|scroll| scroll.cut),
            "no smooth scroll of the strip was cut short: {scrolls:?}"
        );
        fixture.close().await.unwrap();
    });
}
