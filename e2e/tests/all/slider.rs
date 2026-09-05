//! `Slider`, minimally.
//!
//! This fixture exists for one reason: it is the only one that exercises the
//! pointer pass. A template with an unproven pass is not a finished template.

use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::{Fixture, Suite, Viewport, wait};

const VALUE_NOW: &str = "document.querySelector('[role=slider]').getAttribute('aria-valuenow')";

const THUMB: &str = "[role=slider]";

/// The generic battery.
///
/// **Two known defects are visible here, and neither is desired output.**
///
/// * **No `targets()`.** The thumb measures 16x16 and WCAG 2.5.8 wants 24x24
///   (**todo 302**). Asserting it would go red every run for a decision nobody
///   has made, so it is omitted. Add `.targets(THUMB)` once 302 is settled and
///   this test starts guarding it.
/// * **The accessibility baseline contains `tooltip "40"`** - the value bubble
///   is a `role="tooltip"` that nothing references, so assistive technology
///   sees an ownerless tooltip (**todo 309**). It is in the snapshot because a
///   snapshot records what *is*, and a reviewer reading that line needs to know
///   it is a recorded defect rather than the shape we want. When 309 is fixed
///   the baseline changes and that is correct, not a regression.
///
/// This is the cost of snapshots that the contrast pass covers and they do not:
/// a snapshot detects *change*, so a defect present when the baseline was taken
/// is accepted forever unless somebody writes down that it is one. This comment
/// is that writing down.
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

async fn value_now(fixture: &Fixture) -> String {
    fixture
        .page
        .evaluate("document.querySelector('[role=slider]').getAttribute('aria-valuenow')")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}
