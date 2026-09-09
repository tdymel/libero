//! `Tooltip`: all CSS - `:hover`, `:focus-visible` on a direct-child trigger,
//! and the transparent bridge across the gap (todos 405, 406).

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "#save";
const BUBBLE: &str = "#save-tip";
/// Read straight after focus lands: with the theme's 0 ms delays `visibility`
/// flips as soon as the rule matches, so no wait is needed to see it.
const BUBBLE_VISIBILITY: &str = "getComputedStyle(document.querySelector('#save-tip')).visibility";
const WRAPPER_HOVERED: &str = "document.querySelector('#save-tip').parentElement.matches(':hover')";
/// Well outside the tooltip, inside the fixture's padding.
const AWAY: pointer::Point = pointer::Point { x: 2.0, y: 2.0 };

#[test]
fn it_meets_the_baseline() {
    Suite::new("tooltip", "/tooltip")
        .state("focused", &[Step::TabTo(TRIGGER)], BUBBLE)
        .run();
}

async fn visibility(page: &chromiumoxide::Page) -> String {
    page.evaluate(BUBBLE_VISIBILITY)
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

#[test]
fn tab_focus_on_a_direct_child_shows_the_bubble_and_leaving_hides_it() {
    block_on(async {
        let fixture = Fixture::open("/tooltip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        assert_eq!(visibility(page).await, "hidden", "at rest");
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        assert_eq!(visibility(page).await, "visible", "on Tab focus");
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

#[derive(serde::Deserialize)]
struct Gap {
    x: f64,
    y: f64,
    height: f64,
    hit_bubble: bool,
}

/// The pointer rests in the gap between trigger and bubble: the wrapper must
/// still be `:hover`, because the gap is the bubble's own transparent
/// `::before`. Leaving for somewhere else is the control that it can drop.
#[test]
fn hover_shows_the_bubble_and_the_pointer_can_cross_the_gap() {
    block_on(async {
        let fixture = Fixture::open("/tooltip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::hover(page, TRIGGER).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();

        let gap: Gap = page
            .evaluate(
                "(() => { const t = document.querySelector('#save').getBoundingClientRect(); \
                 const b = document.querySelector('#save-tip').getBoundingClientRect(); \
                 const x = t.x + t.width / 2, y = (t.bottom + b.top) / 2; \
                 return { x, y, height: b.top - t.bottom, \
                   hit_bubble: document.elementFromPoint(x, y) === document.querySelector('#save-tip') }; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            gap.height >= 2.0,
            "a {}px gap is no gap to cross",
            gap.height
        );
        assert!(gap.hit_bubble, "the gap should hit the bubble's bridge");

        pointer::move_to(page, pointer::Point { x: gap.x, y: gap.y })
            .await
            .unwrap();
        wait::for_js_true(
            page,
            WRAPPER_HOVERED,
            "the wrapper to stay hovered in the gap",
        )
        .await
        .unwrap();

        pointer::hover(page, BUBBLE).await.unwrap();
        wait::for_js_true(
            page,
            WRAPPER_HOVERED,
            "the wrapper to stay hovered on the bubble",
        )
        .await
        .unwrap();
        assert_eq!(visibility(page).await, "visible", "on the bubble");

        pointer::move_to(page, AWAY).await.unwrap();
        wait::for_js_true(page, &format!("!{WRAPPER_HOVERED}"), "the pointer to leave")
            .await
            .unwrap();
        wait::for_hidden(page, BUBBLE).await.unwrap();

        fixture.console.assert_clean("hovering a tooltip").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The rule the docs state, seen from the wrong side: a trigger one element
/// deep shows on hover but not on Tab, and a debug build warns about it.
#[test]
fn a_wrapped_trigger_shows_on_hover_only_and_warns() {
    block_on(async {
        let fixture = Fixture::open("/tooltip-wrapped", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        assert_eq!(
            visibility(page).await,
            "hidden",
            "Tab focus on a wrapped trigger"
        );

        pointer::hover(page, TRIGGER).await.unwrap();
        wait::for_visible(page, BUBBLE).await.unwrap();

        wait::until("the no-focusable-trigger warning", || async {
            Ok(fixture
                .console
                .peek()
                .iter()
                .any(|message| message.contains("Tooltip: no direct child can take focus")))
        })
        .await
        .unwrap();
        let warnings = fixture.console.drain();
        assert_eq!(warnings.len(), 1, "one warning per mount: {warnings:?}");

        fixture.close().await.unwrap();
    });
}
