//! `Slider`, minimally.
//!
//! This fixture exists for one reason: it is the only one that exercises the
//! pointer pass. A template with an unproven pass is not a finished template.

use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::passes::target_size::MINIMUM;
use e2e::{Fixture, Suite, Viewport, wait};

const VALUE_NOW: &str = "document.querySelector('[role=slider]').getAttribute('aria-valuenow')";

const THUMB: &str = "[role=slider]";

/// The generic battery.
///
/// * **No `targets()`.** The thumb is drawn 16x16, and its 24x24 hit area is a
///   `::before` (todo 302), which a bounding box does not include. That
///   check would report 16 for a thumb that meets 2.5.8, so
///   `the_thumb_takes_the_pointer_over_24px` checks the hit area instead.
///
/// The baseline's `tooltip "40"` is the value bubble, and it is not ownerless:
/// the thumb names it in `aria-describedby` (todo 309). The snapshot prints no
/// descriptions, so `the_thumb_is_described_by_its_value_bubble` checks that.
#[test]
fn it_meets_the_baseline() {
    Suite::new("slider", "/slider").focusable(THUMB).run();
}

#[test]
fn the_thumb_tracks_a_drag() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/slider", viewport).await.unwrap();

            let before = value_now(&fixture).await;

            // Drag right along the track. The intermediate moves are the point:
            // a press followed straight by a release never reaches a component
            // that tracks movement.
            let from = pointer::centre_of(&fixture.page, THUMB).await.unwrap();
            let to = pointer::Point {
                x: from.x + 80.0,
                y: from.y,
            };
            pointer::drag(&fixture.page, from, to, 10).await.unwrap();

            // Waited for, not read: the value is written by a dioxus
            // re-render that the drag does not block on.
            wait::for_js_change(
                &fixture.page,
                VALUE_NOW,
                &before,
                "the slider value to move",
            )
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "at {}: dragging the thumb did not move it: {e}",
                    viewport.name()
                )
            });

            fixture
                .console
                .assert_clean(&format!("a drag at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// The keyboard half of the same contract, which a drag cannot prove.
#[test]
fn the_thumb_moves_with_the_arrow_keys() {
    block_on(async {
        use e2e::passes::keyboard;

        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        keyboard::tab_to(&fixture.page, THUMB, 10).await.unwrap();

        let before = value_now(&fixture).await;
        keyboard::press(&fixture.page, keyboard::ARROW_RIGHT)
            .await
            .unwrap();
        wait::for_js_change(
            &fixture.page,
            VALUE_NOW,
            &before,
            "ArrowRight to move the thumb",
        )
        .await
        .unwrap();

        keyboard::press(&fixture.page, keyboard::HOME)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            &format!("{VALUE_NOW} === '0'"),
            "Home to reach the minimum",
        )
        .await
        .unwrap_or_else(|e| panic!("Home should jump to the minimum: {e}"));

        keyboard::press(&fixture.page, keyboard::END).await.unwrap();
        wait::for_js_true(
            &fixture.page,
            &format!("{VALUE_NOW} === '100'"),
            "End to reach the maximum",
        )
        .await
        .unwrap_or_else(|e| panic!("End should jump to the maximum: {e}"));

        fixture.close().await.unwrap();
    });
}

/// WCAG 2.5.8 on the thumb's hit area, not its box: the thumb is drawn 16px
/// and an invisible square 24px wide takes the pointer (todo 302).
///
/// Hit-tests the square's corners, 11.5px out from the thumb's centre on both
/// axes, and 13px out as the control that the check can fail. Then drags from
/// 11px above the centre, which is outside the thumb's drawn box and outside
/// the slider's root, so only the hit area can start that drag.
#[test]
fn the_thumb_takes_the_pointer_over_24px() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/slider", viewport).await.unwrap();
            let centre = pointer::centre_of(&fixture.page, THUMB).await.unwrap();

            assert_eq!(
                corners_on_thumb(&fixture, centre, MINIMUM / 2.0 - 0.5).await,
                [true; 4],
                "at {}: a corner of the 24px hit area missed the thumb",
                viewport.name()
            );
            assert_eq!(
                corners_on_thumb(&fixture, centre, MINIMUM / 2.0 + 1.0).await,
                [false; 4],
                "at {}: the thumb took the pointer outside its 24px hit area",
                viewport.name()
            );

            let before = value_now(&fixture).await;
            let from = pointer::Point {
                x: centre.x,
                y: centre.y - 11.0,
            };
            let to = pointer::Point {
                x: from.x + 80.0,
                y: from.y,
            };
            pointer::drag(&fixture.page, from, to, 10).await.unwrap();
            wait::for_js_change(
                &fixture.page,
                VALUE_NOW,
                &before,
                "a drag from the hit area to move the thumb",
            )
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "at {}: a drag from 11px above the thumb's centre did not move it: {e}",
                    viewport.name()
                )
            });
            fixture.close().await.unwrap();
        }
    });
}

/// Whether each corner of a square `offset` out from `centre` hit-tests to
/// the thumb.
async fn corners_on_thumb(fixture: &Fixture, centre: pointer::Point, offset: f64) -> Vec<bool> {
    fixture
        .page
        .evaluate(format!(
            "[[-1,-1],[1,-1],[-1,1],[1,1]].map(([dx, dy]) => {{ \
             const el = document.elementFromPoint({x} + dx * {offset}, {y} + dy * {offset}); \
             return !!el && !!el.closest('[role=slider]'); }})",
            x = centre.x,
            y = centre.y,
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// The bubble is a `role="tooltip"`, and a tooltip only means something to
/// assistive technology through the element that points at it.
#[test]
fn the_thumb_is_described_by_its_value_bubble() {
    block_on(async {
        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        let description = e2e::ax::description(&fixture.page, THUMB).await.unwrap();
        assert_eq!(description, "40", "the thumb's computed description");
        fixture.close().await.unwrap();
    });
}

async fn value_now(fixture: &Fixture) -> String {
    fixture
        .page
        .evaluate("document.querySelector('[role=slider]').getAttribute('aria-valuenow')")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}
