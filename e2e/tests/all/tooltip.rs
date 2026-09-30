//! `Tooltip`: a portaled bubble on hover and focus, bridged across the gap (405, 406); it
//! flips at the edge, escapes `overflow: hidden` and closes on Escape (6).

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, linger};
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "#save";
const BUBBLE: &str = "#save-tip";
/// Well outside the tooltip, inside the fixture's padding.
const AWAY: pointer::Point = pointer::Point { x: 2.0, y: 2.0 };

// On `/tooltip/quick`: 10 ms delays, `#before`, `#away`.
const OPEN: &str = "#save-tip:not([hidden])";
/// The touch open delay (libero's `LONG_PRESS`).
const LONG_PRESS_MS: u32 = 500;

async fn is_open<D: Driver>(d: &mut D, open: bool, after: &str) -> Result<()> {
    eventually(d, &format!("{after}: bubble open={open}"), async |d| {
        Ok(d.exists(OPEN).await? == open)
    })
    .await
}

async fn hover_open<D: Driver>(d: &mut D) -> Result<()> {
    ensure!(!d.exists(OPEN).await?, "open at rest");
    d.hover(TRIGGER).await?;
    is_open(d, true, "hovering the trigger").await
}

async fn hover_places_and_leaving_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    hover_open(d).await?;
    let t = d.rect(TRIGGER).await?;
    let b = d.rect(OPEN).await?;
    let gap = b.y - (t.y + t.height);
    ensure!(
        (0.0..=16.0).contains(&gap) && ((b.x + b.width / 2.0) - (t.x + t.width / 2.0)).abs() <= 2.0,
        "trigger {t:?}, bubble {b:?}"
    );
    d.hover("#away").await?;
    is_open(d, false, "leaving").await
}

/// Until no close counts down: the bubble's `data-closing` is gone, or the bubble is.
async fn no_close_pending<D: Driver>(d: &mut D) -> Result<()> {
    eventually(d, "the pending close to clear", async |d| {
        Ok(!d.exists(OPEN).await? || d.attr(OPEN, "data-closing").await?.is_none())
    })
    .await
}

async fn rests_on_the_bubble<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    hover_open(d).await?;
    // The trigger's leave armed the close, the bubble's enter must have cancelled it.
    d.hover(OPEN).await?;
    no_close_pending(d).await?;
    ensure!(d.exists(OPEN).await?, "moving onto the bubble closed it");
    // The mark the wait above relies on, held where the clock can be.
    if d.hold_timers(&[10]).await? {
        d.hover("#away").await?;
        d.settle().await?;
        ensure!(
            d.attr(OPEN, "data-closing").await?.as_deref() == Some("true"),
            "leaving armed no close mark"
        );
        d.fire_timers(10).await?;
    } else {
        d.hover("#away").await?;
    }
    is_open(d, false, "leaving the bubble").await
}

async fn escape_under_the_pointer<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    hover_open(d).await?;
    d.focus("#before").await?;
    d.press(keyboard::ESCAPE).await?;
    is_open(d, false, "Escape").await?;
    eventually_focused(d, "#before", "Escape").await
}

/// Blitz fires no `focusin` for Tab; the silent-focus check opens it (N6).
async fn tab_focus_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#before").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Tab from Before").await?;
    is_open(d, true, "Tab focus").await
}

/// Todo 996, as MUI: on touch a long press opens it, a tap does not, and it
/// closes a while after the release.
async fn long_press_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(!d.exists(OPEN).await?, "open at rest");
    d.long_press(TRIGGER, 800).await?;
    is_open(d, true, "a long press").await?;
    is_open(d, false, "the release").await?;
    // Held only now: the long press above needs the real clock.
    if d.hold_timers(&[LONG_PRESS_MS]).await? {
        d.long_press(TRIGGER, 100).await?;
        d.settle().await?;
        d.fire_timers(LONG_PRESS_MS).await?;
        d.settle().await?;
    } else {
        d.long_press(TRIGGER, 100).await?;
        linger(d, 8).await;
    }
    ensure!(!d.exists(OPEN).await?, "a short touch opened it");
    Ok(())
}

e2e::scenario!(
    hover_opens_it_below_the_trigger_and_leaving_closes_it,
    "/tooltip/quick",
    hover_places_and_leaving_closes,
    android: skip("996: no hover on touch; a long press opens it instead")
);
e2e::scenario!(
    the_pointer_can_rest_on_the_bubble,
    "/tooltip/quick",
    rests_on_the_bubble,
    android: skip("996: no hover on touch; a long press opens it instead")
);
e2e::scenario!(
    a_long_press_opens_it_on_touch,
    "/tooltip/quick",
    long_press_opens,
    native: skip("996: Blitz has no touch input"),
    desktop: skip("1126: no touch input under Xvfb")
);
e2e::scenario!(
    escape_closes_it_under_the_pointer,
    "/tooltip/quick",
    escape_under_the_pointer
);
e2e::scenario!(tab_focus_opens_it, "/tooltip/quick", tab_focus_opens);

#[test]
fn it_meets_the_baseline() {
    Suite::new("tooltip", "/tooltip")
        .state("focused", &[Step::TabTo(TRIGGER)], BUBBLE)
        .run();
}

async fn js<T: serde::de::DeserializeOwned>(page: &chromiumoxide::Page, expression: &str) -> T {
    page.evaluate(expression)
        .await
        .unwrap_or_else(|e| panic!("evaluate {expression}: {e}"))
        .into_value()
        .unwrap_or_else(|e| panic!("read {expression}: {e}"))
}

/// Past the renders and effects a wrong reopen would take.
async fn settle(page: &chromiumoxide::Page) {
    crate::settle::painted(page).await.unwrap();
}

/// Until no close counts down: a wrong one has then fired and taken the bubble.
async fn no_close_pending_on(page: &chromiumoxide::Page) {
    wait::for_js_true(
        page,
        "!document.querySelector('#save-tip')?.hasAttribute('data-closing')",
        "the pending close to clear",
    )
    .await
    .unwrap();
}

#[test]
fn tab_focus_shows_the_bubble_and_leaving_hides_it() {
    block_on(async {
        let fixture = Fixture::open("/tooltip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        assert!(!wait::is_visible(page, BUBBLE).await.unwrap(), "at rest");
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement === document.querySelector('#after')",
            "Tab to move on to the next button",
        )
        .await
        .unwrap();
        wait::for_hidden(page, BUBBLE).await.unwrap();

        fixture
            .console
            .assert_clean("tabbing through a tooltip")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A click focuses the trigger too, and that is not keyboard focus: once the
/// pointer has gone the bubble goes with it.
#[test]
fn a_click_does_not_pin_the_bubble() {
    block_on(async {
        let fixture = Fixture::open("/tooltip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();
        pointer::move_to(page, AWAY).await.unwrap();
        wait::for_hidden(page, BUBBLE).await.unwrap();

        fixture.console.assert_clean("clicking a tooltip").unwrap();
        fixture.close().await.unwrap();
    });
}

#[derive(serde::Deserialize)]
struct Gap {
    x: f64,
    y: f64,
    height: f64,
    hit_bubble: bool,
}

/// The pointer resting in the gap keeps the bubble (its transparent `::before`); leaving
/// elsewhere is the control that it drops.
#[test]
fn hover_shows_the_bubble_and_the_pointer_can_cross_the_gap() {
    block_on(async {
        let fixture = Fixture::open("/tooltip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::hover(page, TRIGGER).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();

        let gap: Gap = js(
            page,
            "(() => { const t = document.querySelector('#save').getBoundingClientRect(); \
             const b = document.querySelector('#save-tip').getBoundingClientRect(); \
             const x = t.x + t.width / 2, y = (t.bottom + b.top) / 2; \
             return { x, y, height: b.top - t.bottom, \
               hit_bubble: document.elementFromPoint(x, y) === document.querySelector('#save-tip') }; })()",
        )
        .await;
        assert!(
            gap.height >= 2.0,
            "a {}px gap is no gap to cross",
            gap.height
        );
        assert!(gap.hit_bubble, "the gap should hit the bubble's bridge");

        pointer::move_to(page, pointer::Point { x: gap.x, y: gap.y })
            .await
            .unwrap();
        no_close_pending_on(page).await;
        assert!(wait::is_visible(page, BUBBLE).await.unwrap(), "in the gap");

        pointer::hover(page, BUBBLE).await.unwrap();
        no_close_pending_on(page).await;
        assert!(
            wait::is_visible(page, BUBBLE).await.unwrap(),
            "on the bubble"
        );

        pointer::move_to(page, AWAY).await.unwrap();
        wait::for_hidden(page, BUBBLE).await.unwrap();

        fixture.console.assert_clean("hovering a tooltip").unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.13: Escape dismisses the bubble without moving the pointer or
/// focus, and it stays dismissed until the pointer or focus comes back.
#[test]
fn escape_closes_it_under_the_pointer_and_on_focus() {
    block_on(async {
        let fixture = Fixture::open("/tooltip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::hover(page, TRIGGER).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, BUBBLE).await.unwrap();
        settle(page).await;
        assert!(
            !wait::is_visible(page, BUBBLE).await.unwrap(),
            "the resting pointer opened it again"
        );
        pointer::move_to(page, AWAY).await.unwrap();
        pointer::hover(page, TRIGGER).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();
        pointer::move_to(page, AWAY).await.unwrap();
        wait::for_hidden(page, BUBBLE).await.unwrap();

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, BUBBLE).await.unwrap();
        settle(page).await;
        assert!(!wait::is_visible(page, BUBBLE).await.unwrap(), "reopened");
        let focused: bool = js(
            page,
            "document.activeElement === document.querySelector('#save')",
        )
        .await;
        assert!(focused, "Escape moved focus off the trigger");

        fixture.console.assert_clean("escaping a tooltip").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A trigger one element deep: focus inside the wrapper opens it too.
#[test]
fn a_wrapped_trigger_shows_on_tab() {
    block_on(async {
        let fixture = Fixture::open("/tooltip-wrapped", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();

        fixture
            .console
            .assert_clean("tabbing to a wrapped trigger")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

#[derive(serde::Deserialize)]
struct Edge {
    below: bool,
    outside_clip: bool,
    hit_bubble: bool,
}

/// A `top` tooltip with no room flips below, and a clipping ancestor does not cut it: the
/// pointer finds the bubble outside the clip box.
#[test]
fn it_flips_at_the_edge_and_escapes_a_clipping_ancestor() {
    block_on(async {
        let fixture = Fixture::open("/tooltip-edge", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();

        let edge: Edge = js(
            page,
            "(() => { const t = document.querySelector('#save').getBoundingClientRect(); \
             const c = document.querySelector('#clip').getBoundingClientRect(); \
             const bubble = document.querySelector('#save-tip'); \
             const b = bubble.getBoundingClientRect(); \
             return { below: b.top >= t.bottom, \
               outside_clip: b.bottom > c.bottom || b.right > c.right, \
               hit_bubble: document.elementFromPoint(b.x + b.width / 2, b.bottom - 2) === bubble }; })()",
        )
        .await;
        assert!(edge.below, "no room above, so the bubble belongs below");
        assert!(
            edge.outside_clip,
            "the bubble should reach past the clip box"
        );
        assert!(edge.hit_bubble, "the part past the clip box is not drawn");

        fixture
            .console
            .assert_clean("a tooltip at the edge")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

#[derive(serde::Deserialize)]
struct Beside {
    on_start: bool,
    token: bool,
    hit_bubble: bool,
}

/// Todo 711: `side: "start"` is the left under LTR and the right under RTL,
/// and the bridge crosses the gap on whichever side it landed.
#[test]
fn a_start_bubble_follows_the_direction() {
    block_on(async {
        for dir in ["ltr", "rtl"] {
            let fixture = crate::rtl_keys::open_in("/tooltip-start", dir).await;
            let page = &fixture.page;

            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            wait::for_visible(page, BUBBLE).await.unwrap();
            let beside: Beside = js(
                page,
                &format!(
                    "(() => {{ const rtl = {rtl}; \
                     const t = document.querySelector('#save').getBoundingClientRect(); \
                     const bubble = document.querySelector('#save-tip'); \
                     const b = bubble.getBoundingClientRect(); \
                     const x = rtl ? (t.right + b.left) / 2 : (b.right + t.left) / 2; \
                     return {{ on_start: rtl ? b.left >= t.right : b.right <= t.left, \
                       token: bubble.matches('[data-state~=\"side-start\"]'), \
                       hit_bubble: document.elementFromPoint(x, t.y + t.height / 2) === bubble }}; }})()",
                    rtl = dir == "rtl"
                ),
            )
            .await;
            assert!(
                beside.on_start,
                "{dir}: the bubble is not on the start side"
            );
            assert!(beside.token, "{dir}: no side-start token");
            assert!(beside.hit_bubble, "{dir}: the gap misses the bridge");

            fixture.console.assert_clean(dir).unwrap();
            fixture.close().await.unwrap();
        }
    });
}
