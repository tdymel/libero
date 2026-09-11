//! `Tooltip`: a portaled bubble opened by hover and keyboard focus, with a
//! transparent bridge across the gap (todos 405, 406), that flips at the
//! viewport edge, escapes an `overflow: hidden` ancestor and closes on Escape
//! (todo 6).

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "#save";
const BUBBLE: &str = "#save-tip";
/// Well outside the tooltip, inside the fixture's padding.
const AWAY: pointer::Point = pointer::Point { x: 2.0, y: 2.0 };

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

/// Long enough for a wrong reopen, a render or two, to have landed.
async fn settle() {
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
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

/// The pointer rests in the gap between trigger and bubble: the bubble stays,
/// because the gap is its own transparent `::before`. Leaving for somewhere
/// else is the control that it can drop.
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
        settle().await;
        assert!(wait::is_visible(page, BUBBLE).await.unwrap(), "in the gap");

        pointer::hover(page, BUBBLE).await.unwrap();
        settle().await;
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
        settle().await;
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
        settle().await;
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

/// A `top` tooltip with no room above flips below its trigger, and a clipping
/// ancestor does not cut it off: the bubble lies outside the clip box and is
/// what the pointer finds there.
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
